import {createBaseURL} from "~/service/account";

export const createWS = async(jwt: string): Promise<WebSocket> => {
    let baseURL = "wss://roullette.botenza.org/api/room";
    // let baseURL = createBaseURL("8081", "ws:");
    const ws = new WebSocket(baseURL + "/ws?jwt=" + jwt)

    return ws
}