import axios, {type AxiosInstance} from "axios";
import type {User} from "~/context/session";
import {createWS} from "~/service/ws";
import {createBaseURL} from "~/service/account";
import { EventEmitter } from 'eventemitter3'
import {braceExpand} from "minimatch";


function callbackToPromise(): any {
    let callback: any;
    const promise = new Promise((resolve, reject) => {
        callback= resolve;
    })

    return {promise, callback}
}

export class RoomService {
    private axiosClient: AxiosInstance;

    private pc: RTCPeerConnection | undefined;

    private ws: WebSocket | undefined;
    private stream?: MediaStream;

    constructor(baseURL?: string) {
        if (!baseURL) {
            baseURL = createBaseURL("8081")
        }
        baseURL = "https://roullette.botenza.org/api/room";


        this.axiosClient = axios.create({
            baseURL: baseURL,
            headers: {'Content-Type': 'application/json'},
        })
    }

    private async createWS(token: string): Promise<WebSocket> {
        if (!this.ws) {
            this.ws = await createWS(token);
            const {callback, promise } = callbackToPromise()
            this.ws.addEventListener("open", callback);
            await promise;

            this.ws!.addEventListener("close", () => {
                this.ws = undefined;
            })
        }

        return this.ws
    }


    async createSession(room: string, user: User) {
        await this.createWS(user.token);
        const pendingCandidates: RTCIceCandidateInit[] = [];

        const emitter = new EventEmitter();
        emitter.addListener("onRemoteDescriptionChanged", async () => {
            // Проверяем, установлен ли remoteDescription
            if (!this.pc?.remoteDescription) {
                console.warn("Remote description not set yet, skipping candidate processing");
                return;
            }

            while (pendingCandidates.length > 0) {
                const candidate = pendingCandidates.shift();
                if (!candidate) continue;

                try {
                    await this.pc.addIceCandidate(candidate);
                    // console.log("Successfully added ICE candidate:", candidate);
                } catch (err) {
                    console.error("Failed to add ICE candidate:", err, candidate);
                    // Можно вернуть кандидат обратно в очередь или обработать ошибку
                    pendingCandidates.unshift(candidate);
                    break;
                }
            }
        })

        this.ws!.addEventListener("message", async (event) => {
            const d = JSON.parse(event.data)
            switch (d.type) {
                case "sdp": {
                    const sdp: RTCSessionDescription = d.playground
                    // 0. Проверка состояния
                    // console.log(this.pc!.signalingState)
                    // if (this.pc!.signalingState !== "stable") {
                    //     await this.pc!.setLocalDescription({ type: "rollback" });
                    // }

                    await this.pc!.setRemoteDescription(sdp);
                    emitter.emit("onRemoteDescriptionChanged", sdp);

                    const answer = await this.pc!.createAnswer()
                    await this.sendAnswer(user, answer, room)
                    await this.pc!.setLocalDescription(answer);
                    break
                }
                case "candidate": {
                    if (d.playground) {
                        if (this.pc!.remoteDescription) {
                            await this.pc!.addIceCandidate(d.playground);
                        } else {
                            pendingCandidates.push(d.playground);
                        }
                    }
                    break
                }
            }
        })

        let stream: MediaStream;
        if (!this.pc) {
            this.pc = new RTCPeerConnection({
                iceServers: [
                    {
                        urls: ['stun:stun.l.google.com:19302', 'stun:stun.l.google.com:5349', 'stun:stun1.l.google.com:3478']
                    }
                ]
            });

            this.pc!.addEventListener("connectionstatechange", (e) => {
                if (["closed", "failed"].includes(this.pc!.connectionState)) {
                    this.pc = undefined;
                    emitter.emit("closed", e)
                }
            })

            this.pc!.onicecandidate = async (event) => {
                if (!event.candidate || this.pc!.iceConnectionState === "connected") {
                    return
                }
                await this.candidate(event.candidate, user, room);
            }

            this.pc!.onnegotiationneeded = async (e) => {
                console.log('negotiationneeded', e)
                await this.renegotiation(false, room, user);
            }

            const stream = await navigator.mediaDevices.getUserMedia({video: true, audio: false})
            stream.getTracks().forEach(track => this.pc!.addTrack(track, stream));

            this.stream = stream;
        } else {
            await this.renegotiation(true, room, user);
        }

        // pc.addTransceiver('video')


        return {pc: this.pc!, localStream: this.stream!, emitter: emitter}
    }

    private async candidate(candidate: RTCIceCandidate, user: User, room: string) {
        const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/candidate", {candidate, room_id: room}, {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${user.token}`,
            }
        });
        return resp.data.answer
    }

    private async sendAnswer(user: User, answer: RTCSessionDescriptionInit, room_id: string) {
        const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/answer", {answer, room_id: room_id}, {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${user.token}`,
            }
        });
        return resp.data.answer
    }


    private async createOffer(iceRestart: boolean, room_id: string, user: User) {
        const offer = await this.pc?.createOffer({iceRestart})
        try {
            await this.pc?.setLocalDescription(offer)
        } catch (error) {
            console.error(error)
            return
        }
        const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/offer", {offer, room_id}, {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${user.token}`,
            }
        });
        await this.pc?.setRemoteDescription(resp.data.answer)
        return resp.data.answer
    }

    private async renegotiation(iceRestart: boolean, room: string, user: User) {
        await this.createOffer(iceRestart, room, user);
    }
}

export const roomService = new RoomService();