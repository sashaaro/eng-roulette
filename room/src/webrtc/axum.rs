use std::collections::HashMap;
use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::ws::{Message, WebSocket};
use axum::routing::{any, post};
use futures::executor::block_on;
use futures::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use crate::webrtc::sfu::{Signalling, SFU};
use crate::webrtc::types::{AnswerResponse, CandidatesRequest};
use futures::{SinkExt, StreamExt};

pub(crate) type SocketClient = (Mutex<SplitSink<WebSocket, Message>>, Mutex<SplitStream<WebSocket>>);

#[derive(Clone)]
pub struct AppState {
    pub(crate) sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
    pub(crate) sfu: SFU,
}


struct WebsocketSignalling {
    sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>
}

impl WebsocketSignalling {
    fn new(sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>) -> Self {
        Self {
            sessions,
        }
    }
}

impl Signalling for WebsocketSignalling {
    fn send_sdp(&self, session_id: String, sdp: RTCSessionDescription) -> Pin<Box<dyn Future<Output=()> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id);
            if session.is_none() {
                println!("session not found: {}", session_id); // todo log
            } else {
                let session = session.unwrap();
                let playground = serde_json::to_string(&sdp).unwrap(); // TODO
                session.0.lock().await.send(Message::from(playground)).await.unwrap();// TODO
            }
        })
    }
}

pub async fn create_sfu_router() -> Router {
    let sessions = Arc::new(Mutex::new(HashMap::new()));

    let signalling = Box::new(WebsocketSignalling::new(
        Arc::clone(&sessions),
    ));
    let state = AppState {
        sfu: SFU::new(signalling),
        sessions: Arc::clone(&sessions),
    };

    let app = Router::new()
        .route("/ws", any(ws))
        .route("/accept-offer", post(accept_offer))
        .route("/candidates", post(candidates))
        .layer(CorsLayer::permissive())
        .with_state(state);

    app
}

#[derive(Deserialize)]
struct WsRequest {
    session_id: String,
}


async fn ws(
    ws: WebSocketUpgrade,
    Query(req): Query<WsRequest>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {

    let sfu = app_state.sfu.clone();
    let sessions = app_state.sessions.clone();

    let session_id = req.session_id.clone();

    ws.on_failed_upgrade(move |error| {
        println!("on_failed_upgrade {:?}", error);
        block_on(sessions.lock()).remove(&req.session_id);
    })
        .on_upgrade(async move |socket| {
            let (sender, receiver) = socket.split();
            let socket_client: Arc<SocketClient> = Arc::new((
                Mutex::new(sender),
                Mutex::new(receiver)
            ));

            app_state
                .sessions
                .lock()
                .await
                .insert(session_id.clone(), Arc::clone(&socket_client));

            handle_socket(
                sfu,
                session_id.clone(),
                Arc::downgrade(&socket_client),
            ).await;
        })
}

async fn handle_socket(
    sfu: SFU,
    session_id: String,
    socket_client: Weak<SocketClient>,
) {
    let sfu = sfu.clone();

    tokio::spawn(async move {
        loop {
            let socket_client = socket_client.upgrade();
            if socket_client.is_none() {
                break;
            }
            let socket_client = socket_client.unwrap();
            let mut stream = socket_client.1.lock().await;
            let msg = stream.next().await;

            if let Some(result) = msg {
                if result.is_ok() {
                    let msg = result.unwrap();
                    if process_message(sfu.clone(), msg, session_id.clone())
                        .await
                        .is_break()
                    {
                        return;
                    }
                } else {
                    // TODO result.err().unwrap();
                }

            } else {
                println!("client {session_id} abruptly disconnected");
                return;
            }
        }
    });
}

async fn process_message(
    sfu: SFU,
    msg: Message,
    session_id: String,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let sdp = serde_json::from_str::<RTCSessionDescription>(&t).unwrap();
            if (sdp.sdp_type == RTCSdpType::Answer) {
                let peer_connection = sfu.peers.lock().await.get(&session_id).unwrap().clone();

                peer_connection.rtp_peer.set_remote_description(sdp).await.unwrap();
            }
        }
        Message::Binary(d) => {
            println!(">>> sent {} bytes: {:?}", d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    session_id, cf.code, cf.reason
                );
            } else {
                println!(">>> {session_id} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {session_id} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {session_id} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

#[derive(Deserialize, Serialize)]
struct AcceptOfferReq {
    offer: RTCSessionDescription,
    session_id: String,
    room_id: String,
}


async fn accept_offer(State(app_state): State<AppState>, Json(req): Json<AcceptOfferReq>) -> impl IntoResponse {
    let answer = app_state.sfu.accept_offer(req.session_id.clone(), req.offer, req.room_id).await;

    serde_json::to_string(&AnswerResponse {
        answer,
        session_id: req.session_id,
    })
        .unwrap()
}

async fn candidates(
    State(app_state): State<AppState>,
    Json(candidates_req): Json<CandidatesRequest>,
) -> impl IntoResponse {
    if candidates_req.candidates.len() == 0 {
        return "invalid candidates";
    }

    if candidates_req.session_id.len() == 0 {
        return "invalid candidates";
    }

    let sessions = Arc::clone(&app_state.sessions);
    let sessions = sessions.lock().await;
    let socket_client = sessions.get(&candidates_req.session_id);

    if socket_client.is_none() {
        return "";
    }

    let sfu = app_state.sfu.clone();
    let peers = sfu.peers.lock().await;
    let peer = peers.get(&candidates_req.session_id);

    if peer.is_none() {
        return "no peer";
    }
    let peer = peer.unwrap();

    // let peer_connection = app_state
    //     .sessions
    //     .lock()
    //     .await
    //     .get(&candidates_req.session_id)
    //     .unwrap()
    //     .clone();

    let peer = Arc::clone(&peer);
    candidates_req
        .candidates
        .iter()
        .for_each(|c| {
            block_on(peer.rtp_peer.add_ice_candidate(c.clone())).unwrap()
        });

    "ok"
}
