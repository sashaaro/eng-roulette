
export const createWS = async(jwt: string): Promise<WebSocket> => {
    const ws = new WebSocket("ws://localhost:8080/ws?session_id=" + jwt)

    return ws
}