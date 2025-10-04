import axios, {type AxiosInstance} from "axios";
import type {User} from "./../context/session.tsx";
import { EventEmitter } from 'eventemitter3'
import config from "./config.ts";

interface WebSocketMessage {
    type: 'sdp' | 'candidate';
    playground?: RTCSessionDescriptionInit | RTCIceCandidateInit;
}

interface PromiseWithCallback<T = void> {
    promise: Promise<T>;
    callback: (value: T) => void;
}

function createPromiseWithCallback<T = void>(): PromiseWithCallback<T> {
    let callback: (value: T) => void;
    const promise = new Promise<T>((resolve) => {
        callback = resolve;
    });

    return { promise, callback: callback! };
}

const ICE_SERVERS = [
    {
        urls: ['stun:stun.l.google.com:19302', 'stun:stun.l.google.com:5349', 'stun:stun1.l.google.com:3478']
    }
];

const MEDIA_CONSTRAINTS = {
    video: true,
    audio: false
};

export interface WebrtcSession {
    pc: RTCPeerConnection
    localStream: MediaStream;
    emitter: EventEmitter;
}

export class RoomService {
    private axiosClient: AxiosInstance;

    private pc: RTCPeerConnection | undefined;

    private ws: WebSocket | undefined;
    private stream?: MediaStream;

    constructor(baseURL: string) {
        this.axiosClient = axios.create({
            baseURL: baseURL,
            headers: {'Content-Type': 'application/json'},
        })
    }

    private async createWS(token: string): Promise<WebSocket> {
        if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
            const wsUrl = `${config.roomWS}?jwt=${token}`;
            this.ws = new WebSocket(wsUrl);

            const { callback, promise } = createPromiseWithCallback<Event>();
            this.ws.addEventListener("open", callback);

            this.ws.addEventListener("error", (error) => {
                console.error("WebSocket error:", error);
                this.ws = undefined;
            });

            this.ws.addEventListener("close", (event) => {
                console.log("WebSocket closed:", event.code, event.reason);
                this.ws = undefined;
            });

            await promise;
        }

        return this.ws;
    }


    async createSession(room: string, user: User): Promise<WebrtcSession> {
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
            try {
                const message: WebSocketMessage = JSON.parse(event.data);
                await this.handleWebSocketMessage(message, user, room, emitter, pendingCandidates);
            } catch (error) {
                console.error("Error handling WebSocket message:", error);
            }
        });

        if (!this.pc) {
            await this.initializePeerConnection(user, room, emitter);
        } else {
            await this.renegotiation(true, room, user);
        }

        return { pc: this.pc!, localStream: this.stream!, emitter };
    }

    private async candidate(candidate: RTCIceCandidate, user: User, room: string): Promise<RTCSessionDescription> {
        try {
            const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/candidate", {
                candidate,
                room_id: room
            }, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${user.token}`,
                }
            });
            return resp.data.answer;
        } catch (error) {
            console.error("Error sending candidate:", error);
            throw error;
        }
    }

    private async sendAnswer(user: User, answer: RTCSessionDescriptionInit, room_id: string): Promise<RTCSessionDescription> {
        try {
            const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/answer", {
                answer,
                room_id
            }, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${user.token}`,
                }
            });
            return resp.data.answer;
        } catch (error) {
            console.error("Error sending answer:", error);
            throw error;
        }
    }


    private async createOffer(iceRestart: boolean, room_id: string, user: User): Promise<RTCSessionDescription | undefined> {
        if (!this.pc) {
            console.error("Peer connection not initialized");
            return;
        }

        try {
            const offer = await this.pc.createOffer({ iceRestart });
            await this.pc.setLocalDescription(offer);

            const resp = await this.axiosClient.post<{answer: RTCSessionDescription}>("/offer", {
                offer,
                room_id
            }, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${user.token}`,
                }
            });

            await this.pc.setRemoteDescription(resp.data.answer);
            return resp.data.answer;
        } catch (error) {
            console.error("Error creating offer:", error);
            throw error;
        }
    }

    private async renegotiation(iceRestart: boolean, room: string, user: User) {
        await this.createOffer(iceRestart, room, user);
    }

    private async initializePeerConnection(user: User, room: string, emitter: EventEmitter): Promise<void> {
        try {
            this.pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });

            this.setupPeerConnectionEventHandlers(user, room, emitter);
            await this.setupMediaStream();

            this.setupWindowUnloadHandler();
        } catch (error) {
            console.error("Error initializing peer connection:", error);
            throw error;
        }
    }

    private setupPeerConnectionEventHandlers(user: User, room: string, emitter: EventEmitter): void {
        if (!this.pc) return;

        this.pc.addEventListener("connectionstatechange", (event) => {
            if (!this.pc) return;

            console.log("Connection state changed:", this.pc.connectionState);
            if (["closed", "failed"].includes(this.pc.connectionState)) {
                this.pc = undefined;
                emitter.emit("closed", event);
            }
        });

        this.pc.onicecandidate = async (event) => {
            if (!event.candidate || !this.pc || this.pc.iceConnectionState === "connected") {
                return;
            }

            try {
                await this.candidate(event.candidate, user, room);
            } catch (error) {
                console.error("Error sending ICE candidate:", error);
            }
        };

        this.pc.onnegotiationneeded = async () => {
            try {
                await this.renegotiation(false, room, user);
            } catch (error) {
                console.error("Error during renegotiation:", error);
            }
        };
    }

    private async setupMediaStream(): Promise<void> {
        try {
            const stream = await navigator.mediaDevices.getUserMedia(MEDIA_CONSTRAINTS);
            stream.getTracks().forEach(track => {
                if (this.pc) {
                    this.pc.addTrack(track, stream);
                }
            });
            this.stream = stream;
        } catch (error) {
            console.error("Error accessing media devices:", error);
            throw error;
        }
    }

    private setupWindowUnloadHandler(): void {
        window.addEventListener("beforeunload", () => {
            console.log("Closing peer connection and WebSocket");
            this.pc?.close();
            this.ws?.close();
        });
    }

    private async handleWebSocketMessage(
        message: WebSocketMessage,
        user: User,
        room: string,
        emitter: EventEmitter,
        pendingCandidates: RTCIceCandidateInit[]
    ): Promise<void> {
        if (!this.pc) {
            console.error("Peer connection not initialized");
            return;
        }

        switch (message.type) {
            case "sdp":
                await this.handleSdpMessage(message.playground as RTCSessionDescriptionInit, user, room, emitter);
                break;
            case "candidate":
                await this.handleCandidateMessage(message.playground as RTCIceCandidateInit, pendingCandidates);
                break;
            default:
                console.warn("Unknown message type:", message.type);
        }
    }

    private async handleSdpMessage(
        sdp: RTCSessionDescriptionInit,
        user: User,
        room: string,
        emitter: EventEmitter
    ): Promise<void> {
        if (!this.pc || !sdp) {
            console.error("Invalid SDP message or peer connection not initialized");
            return;
        }

        try {
            await this.pc.setRemoteDescription(sdp);
            emitter.emit("onRemoteDescriptionChanged", sdp);

            const answer = await this.pc.createAnswer();
            await this.sendAnswer(user, answer, room);
            await this.pc.setLocalDescription(answer);
        } catch (error) {
            console.error("Error handling SDP message:", error);
        }
    }

    private async handleCandidateMessage(
        candidate: RTCIceCandidateInit,
        pendingCandidates: RTCIceCandidateInit[]
    ): Promise<void> {
        if (!candidate) {
            return;
        }

        if (!this.pc) {
            console.error("Peer connection not initialized");
            return;
        }

        try {
            if (this.pc.remoteDescription) {
                await this.pc.addIceCandidate(candidate);
            } else {
                pendingCandidates.push(candidate);
            }
        } catch (error) {
            console.error("Error handling ICE candidate:", error);
        }
    }
}

export const roomService = new RoomService(config.roomURL);