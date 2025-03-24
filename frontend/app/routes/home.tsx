import type { Route } from "./+types/home";
import {useAuth} from "~/context/session";
import {Link} from "react-router";
import {useForm} from "react-hook-form";
import {useCallback, useEffect, useState} from "react";
import {roomService} from "~/service/room";

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

    const [tracks, setTracks] = useState<Array<MediaStreamTrack>>([])

    const {user} = useAuth()

    const onTrack = useCallback((track: RTCTrackEvent) => {
        console.log(track)
        setTracks([...tracks, track.track])
    }, [tracks])

    const onSubmit = async (data: Inputs) => {
        const {pc, localStream} = await roomService.createSession(data.room_name, user!)
        pc.ontrack = onTrack

        console.log(pc)
    }

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="text-center p-4">
            <input type="text" {...register("room_name")}/>
            <button type="submit">Join to room</button>

            <div>
                {tracks.map((track, i) => (
                    <div key={i}>
                        <video muted src={track}></video>
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
