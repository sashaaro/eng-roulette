import {PureComponent, createRef} from 'react'

export interface VideoSrcProps {
    id?: string
    autoPlay: boolean
    src?: string
    srcObject: MediaStream | null
    muted?: boolean
    objectFit?: string
    mirrored?: boolean
}

export default class VideoSrc extends PureComponent<VideoSrcProps> {
    videoRef = createRef<HTMLVideoElement>()

    componentDidMount () {
        this.componentDidUpdate()

        // this.videoRef.current!.onresize = e => {
        //     const el = e.target as HTMLVideoElement
        //     this.maybeTriggerResize(el)
        // }
    }
    componentDidUpdate() {
        const { srcObject, src } = this.props
        const muted = !!this.props.muted

        const video = this.videoRef.current

        if (video) {
            if ('srcObject' in video as unknown) {
                if (video.srcObject !== srcObject) {
                    video.srcObject = srcObject
                }
            } else if (video.src !== src) {
                video.src = src || ''
            }

            // Rather than setting muted property in <video> directly, we set it here
            // to fix some issues in tests. For more details see commit 4b3cf45bf.
            video.muted = muted

            video.style.objectFit = this.props.objectFit || ''
        }
    }

    render() {
        return (
            <video
                id={this.props.id}
                autoPlay={this.props.autoPlay}
                playsInline={true}
                ref={this.videoRef}
            />
        )
    }
}