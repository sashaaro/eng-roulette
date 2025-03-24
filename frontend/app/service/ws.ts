
export const createWS = async(jwt: string): Promise<WebSocket> => {
    const ws = new WebSocket("ws://localhost:8081/ws?jwt=" + jwt)

    return ws
}