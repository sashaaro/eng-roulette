import {useForm} from "react-hook-form";
import {useCallback, useState} from "react";
import {roomService, type WebrtcSession} from "./../service/room.ts";
import {useAuth} from "./../context/session.tsx";
import VideoSrc from "./../component/VideoSrc.tsx";

type Inputs = {
    room_name: string;
}

export function JoinRoom({}) {
    const {
        register,
        handleSubmit,
    } = useForm<Inputs>({defaultValues: {room_name: "default"}});

    const [tracks, setTracks] = useState<Array<RTCTrackEvent>>([])
    const [localSteam, setLocalStream] = useState<MediaStream>()
    const [webrtcSession, setWebrtcSession] = useState<WebrtcSession>()
    const {user} = useAuth()

    const onTrack = useCallback((track: RTCTrackEvent) => {
        track.streams.forEach((s) => {
            console.log(s.getTracks())
            s.getTracks()[0].addEventListener("ended", () => {
                const index = tracks.indexOf(track);
                setTracks([...tracks.slice(0, index), ...tracks.slice(index + 1)]);
            })
        })

        setTracks((tracks) => {
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
            setWebrtcSession({...webrtcSession}) // trigger render
        })
    }, [user])

    const connected = (webrtcSession?.pc.connectionState === "connected" || webrtcSession?.pc.connectionState === "connecting")
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
    <div className="local-stream">
        {localSteam ? <VideoSrc autoPlay={true} srcObject={localSteam}/> : null}
        </div>
        {tracks.map((track, i) => (
            <VideoSrc key={i} muted srcObject={track.streams[0]} autoPlay={true}/>
        ))}
        </div> : null}
        </div>
    );
}