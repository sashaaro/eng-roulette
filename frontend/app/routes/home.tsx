import type { Route } from "./+types/home";
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
    const {user} = useAuth()

    const [loading, setLoading] = useState(false)

    const onTrack = useCallback((track: RTCTrackEvent) => {
        track.streams.forEach((s) => {
            s.addEventListener("removetrack", () => {
                console.log("remove track")
                const index = tracks.indexOf(track);
                setTracks([...tracks.slice(0, index), ...tracks.slice(index + 1)]);
            })
        })

        setTracks([...tracks, track])
    }, [tracks])

    const onSubmit = async (data: Inputs) => {
        setLoading(true)

        let pc: RTCPeerConnection
        let localStream: MediaStream
        try {
            const res = await roomService.createSession(data.room_name, user!)
            pc = res.pc
            localStream = res.localStream
        } catch (err) {
            alert(err)
            setLoading(false)
            return
        }
        pc.ontrack = onTrack

        setLoading(false)
    }

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="text-center p-4">
            <input type="text" {...register("room_name")}/>
            <button type="submit" disabled={loading}>Join to room</button>

            <div>
                {tracks.map((track, i) => (
                    <div key={i}>
                        <VideoSrc muted srcObject={track.streams[0]} autoPlay={true}/>
                    </div>
                ))}
            </div>
        </form>
    );
}

export default function Home({ loaderData }: Route.ComponentProps) {
    const {user} = useAuth();

    return (
        <div className="text-center p-4">
            <h1 className="text-3xl font-bold underline">
                Home
                {user ? <div>Hi {user.username}</div> : null}
            </h1>
            {user ? <JoinRoom/> : <nav>
                <Link to={"/login"}>Login</Link>
                <Link to={"/register"}>Register</Link>
            </nav>}

        </div>
    );
}
