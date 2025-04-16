const config = {
    accountURL: import.meta.env.VITE_ACCOUNT_API || "http://localhost:8081",
    roomURL: import.meta.env.VITE_ROOM_API || "http://localhost:8082",
    roomWS: ""
}

const roomUrl = new URL(config.roomURL);
config.roomWS = (roomUrl.protocol === "https" ? "wss" : "ws") + ":" + roomUrl.host


export default config;