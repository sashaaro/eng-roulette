import {createBaseURL} from "~/service/account";

export const createWS = async(baseURL: string, jwt: string): Promise<WebSocket> => {
    if (!baseURL) {
        baseURL = createBaseURL("8081", "ws:");
    }
    return new WebSocket(baseURL + "/ws?jwt=" + jwt)
}