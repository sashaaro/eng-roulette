import axios, {type AxiosInstance} from "axios";
import type {User} from "~/context/session";
import {createWS} from "~/service/ws";
import {createBaseURL} from "~/service/account";


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

    async createSession(room: string, user: User) {
        this.ws = await createWS(user.token);

        const {callback, promise } = callbackToPromise()
        this.ws.onopen = callback
        await promise;

        this.ws.addEventListener("message", async (event) => {
            const d = JSON.parse(event.data)
            switch (d.type) {
                case "sdp": {
                    await this.pc!.setRemoteDescription(d.playground);
                    const answer = await this.pc!.createAnswer()
                    await this.sendAnswer(user, answer, room)
                    await this.pc!.setLocalDescription(answer);
                    break
                }
                case "candidate": {
                    if (d.playground) {
                        await this.pc!.addIceCandidate(d.playground)
                    }
                    break
                }
            }
        })

        this.pc = new RTCPeerConnection({
            iceServers: [
                {
                    urls: ['stun:stun.l.google.com:19302', 'stun:stun.l.google.com:5349', 'stun:stun1.l.google.com:3478']
                }
            ]
        });

        this.pc!.onconnectionstatechange = (e) => {
            if (["closed", "failed", "disconnected"].includes(this.pc!.connectionState)) {
                this.pc = undefined;
            }
        }

        this.pc!.onicecandidate = async (event) => {
            if (!event.candidate) {
                return
            }
            await this.candidate(event.candidate, user, room);
        }

        this.pc!.onnegotiationneeded = e => {
            console.log('negotiationneeded', e)
            this.renegotiation(room, user);
        }

        // pc.addTransceiver('video')

        const stream = await navigator.mediaDevices.getUserMedia({video: true, audio: false})
        stream.getTracks().forEach(track => this.pc!.addTrack(track, stream));

        return {pc: this.pc!, localStream: stream}
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


    private async createOffer(room_id: string, user: User) {
        const offer = await this.pc?.createOffer()
        await this.pc?.setLocalDescription(offer)
        const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/offer", {offer, room_id}, {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${user.token}`,
            }
        });
        await this.pc?.setRemoteDescription(resp.data.answer)
        return resp.data.answer
    }

    private async renegotiation(room: string, user: User) {
        await this.createOffer(room, user);
    }
}

export const roomService = new RoomService();