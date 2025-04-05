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

    const connected = (connState === "connected" || connState === "connecting")
    return (
        <div className="join-room">
            {connected ? null : <form onSubmit={handleSubmit(onSubmit)} className="join-room-form p-4">
                <div>
                    <label htmlFor="room_name">Room Name</label>
                    <input type="text" className="form-control" {...register("room_name")}/>
                </div>
                <div>
                    <button type="submit" className="btn btn-success">Join to room</button>
                </div>
            </form>}
            {connected ? <div className="room">
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
            </div> : null}
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
        <div>
            <ul className="nav flex-column">
                <li className="nav-item"><Link className="nav-link" to={"/login"}>Login</Link></li>
                <li className="nav-item"><Link className="nav-link" to={"/register"}>Register</Link></li>
                {user ? <li className="nav-item"><a className="nav-link" href="#" onClick={onExit}>Exit</a></li>: null}
            </ul>
            <div>
                {user ? <div>
                    <div className="text-center h3">Hi {user.username}</div>
                    <JoinRoom/>
                </div> : null}
            </div>
        </div>
    );
}
