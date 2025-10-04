const config = {
    accountURL: import.meta.env.VITE_ACCOUNT_API || location.origin + "/api/account",
    roomURL: import.meta.env.VITE_ROOM_API || location.origin + "/api/room",
    roomWS: ""
}

const roomUrl = new URL(config.roomURL);
config.roomWS = (roomUrl.protocol === "https:" ? "wss" : "ws") + ":" + roomUrl.hostname + roomUrl.pathname + "/ws";
console.log(roomUrl, config.roomWS)

export default config;