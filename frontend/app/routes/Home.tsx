import type { Route } from "./+types/Home";
import {useAuth} from "~/context/session";
import {Link} from "react-router";
import {useForm} from "react-hook-form";
import {useCallback, useEffect, useState} from "react";
import {roomService} from "~/service/room";
import VideoSrc from "~/component/VideoSrc";

export function loader() {
  return { name: "React Router" };
}

type Inputs = {
    room_name: string;
}

function JoinRoom({}) {
    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
        setValue
    } = useForm<Inputs>()

    useEffect(() => {
        setValue("room_name", "default") // initial value
    }, [])

    const [tracks, setTracks] = useState<Array<RTCTrackEvent>>([])
    const [localSteam, setLocalStream] = useState<MediaStream>()
    const {user} = useAuth()

    const [connState, setConnState] = useState<RTCPeerConnectionState>("new")

    const onTrack = useCallback((track: RTCTrackEvent) => {
        track.streams.forEach((s) => {
            console.log(s.getTracks())
            s.getTracks()[0].addEventListener("ended", () => {
                console.log("onended");
                const index = tracks.indexOf(track);
                setTracks([...tracks.slice(0, index), ...tracks.slice(index + 1)]);
            })
        })

        setTracks((tracks) => {
            console.log("onTrack", tracks)
            return [...tracks, track]
        } )
    }, [])

    const onSubmit = useCallback(async (data: Inputs) => {
        let webrtcSession
        try {
            webrtcSession = await roomService.createSession(data.room_name, user!)
            setLocalStream(webrtcSession.localStream)
        } catch (err) {
            alert(err)
            return
        }
        webrtcSession.pc.ontrack = onTrack
        webrtcSession.pc.addEventListener("connectionstatechange", () => {
            console.log("onconnectionstatechange " + webrtcSession.pc.connectionState)
            setConnState(webrtcSession.pc.connectionState)
        })
        // webrtcSession.emitter.once("closed", () => {
        //     setConnState(false);
        //     setTracks([]);
        // })
    }, [])

    useEffect(() => {

        return () => {

        }
    }, []);

    const connected = (connState === "connected" || connState === "connecting") && tracks.length > 0
    return (
        <div>
            {connected ? null : <form onSubmit={handleSubmit(onSubmit)} className="text-center p-4">
                <input type="text" {...register("room_name")}/>
                <button type="submit">Join to room</button>
            </form>}
            <div style={{
                position: 'relative',
                minHeight: '500px',
                background: "black",
                borderRadius: "15px",
                display: connected ? "block" : "none",
            }}>
                <div style={{
                    width: "140px",
                    position: "absolute",
                    zIndex: "10",
                    right: "10px",
                    bottom: "10px",
                    borderRadius: "15px",
                }}>
                    {localSteam ? <VideoSrc autoPlay={true} srcObject={localSteam}/> : null}
                </div>
                {tracks.map((track, i) => (
                    <div key={i}>
                        <VideoSrc muted srcObject={track.streams[0]} autoPlay={true}/>
                    </div>
                ))}
            </div>
        </div>
    );
}

export default function Home({ loaderData }: Route.ComponentProps) {
    const {user, setUser} = useAuth();

    const onExit = useCallback((e) => {
        e.preventDefault();
        setUser(null);
        localStorage.removeItem("session"); // TODO refactoring
        // TODO roomService.ws.close();
    }, [])

    return (
        <div className="text-center p-4">
            <h1 className="text-3xl font-bold underline">
                {user ?
                    <div>
                        <div title={user.id + ''}>Hi {user.username}</div><a href="#" onClick={onExit}>Exit</a>
                    </div>
                    : null}
            </h1>
            {user ? <JoinRoom/> : <nav>
                <div><Link to={"/login"}>Login</Link></div>
                <div><Link to={"/register"}>Register</Link></div>
            </nav>}

        </div>
    );
}
