use std::collections::HashMap;
use std::sync::{Arc, Weak};

use crate::webrtc::types::{AnswerResponse, CandidatesRequest};
use anyhow::Result;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    Json, Router,
};
use futures::executor::block_on;
use futures::stream::{SplitSink, SplitStream};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::ops::{ControlFlow, Deref};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tower_http::cors::CorsLayer;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocalWriter;
use webrtc::track::track_remote::TrackRemote;
use webrtc::Error;
// fn generate_random_string(length: usize) -> String {
//     const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
//                              abcdefghijklmnopqrstuvwxyz\
//                              0123456789";
//     let mut rng = rand::thread_rng();
//
//     let random_string: String = (0..length)
//         .map(|_| {
//             let idx = rng.gen_range(0..CHARSET.len());
//             CHARSET[idx] as char
//         })
//         .collect();
//
//     random_string
// }

type SocketClient = Arc<(SplitSink<WebSocket, Message>, SplitStream<WebSocket>)>;

struct Peer {
    session_id: String,
    rtp_peer: RTCPeerConnection,
    remote_tracks: Vec<TrackRemote>,
    //local_tracks: Vec<Box<dyn TrackLocal>>,
}

struct SFU {
    rooms: Mutex<HashMap<String, Arc<Mutex<HashMap<String, Arc<Peer>>>>>>,
    peers: Mutex<HashMap<String, Arc<Peer>>>,
}

struct SfuService(Arc<(SFU)>);

impl Clone for SfuService {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for SfuService {
    type Target = SFU;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SfuService {
    async fn new_peer(&mut self, session_id: String, room_id: String) -> Arc<Peer> {
        let mut rooms = self.rooms.lock().await;
        if !rooms.contains_key(&room_id) {
            rooms.insert(room_id.clone(), Default::default());
        }
        let room = rooms.get(&room_id).unwrap();

        let mut room = room.lock().await;

        let mut peer: Arc<Peer>;
        if room.contains_key(&session_id) {
            peer = room.get(&session_id).unwrap().clone();

            return peer;
        }

        peer = Arc::new(Peer {
            session_id: session_id.clone(),
            rtp_peer: create_peer().await.unwrap(),
            remote_tracks: vec![],
            //local_tracks: vec![],
        });

        room.insert(peer.session_id.clone(), Arc::clone(&peer));

        peer.rtp_peer
            .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
                println!("Peer Connection on ice candidate: {:?}", c);

                Box::pin(async {})
            }));

        let peer2 = Arc::clone(&peer);
        peer.rtp_peer.on_negotiation_needed(Box::new(move || {
            Box::pin({
                let peer2 = Arc::clone(&peer2);
                async move {
                    let sdp = peer2.rtp_peer.create_offer(None).await.unwrap();
                    peer2
                        .rtp_peer
                        .set_local_description(sdp.clone())
                        .await
                        .unwrap();
                    // other_conn.send(Message::from(serde_json::to_string(&sdp).unwrap())).unwrap();
                }
            })
        }));


        let peer2 = Arc::clone(&peer);
        let sfu = self.clone();
        Arc::clone(&peer).rtp_peer.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {

            println!("Peer Connection State has changed: {s}");
            match s {
                RTCPeerConnectionState::Closed
                | RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Failed => {
                    block_on(sfu.rooms.lock()).remove(peer2.session_id.as_str());
                    block_on(sfu.peers.lock()).remove(peer2.session_id.as_str());
                }
                RTCPeerConnectionState::New => {
                    block_on(sfu.peers.lock()).insert(peer2.session_id.clone(), Arc::clone(&peer2));
                }
                _ => {}
            }
            Box::pin(async {})
        },
        ));


        let sfu = self.clone();
        let room_id = room_id.clone();

        let o_peer = Arc::clone(&peer);
        let peer = Arc::clone(&peer);
        Arc::clone(&peer).rtp_peer.on_track(Box::new(move |track, _, _| {
            let weak_peer = Arc::downgrade(&peer);

            let sfu = sfu.clone();

            let room_id = room_id.clone();
            // peer2.local_tracks.insert(session_id.clone(), track.clone());
            let sfu = sfu.clone();

            let media_ssrc = track.ssrc();
            tokio::spawn(async move {
                let mut result = Result::<usize>::Ok(0);
                while result.is_ok() {
                    let timeout = tokio::time::sleep(Duration::from_secs(3));
                    tokio::pin!(timeout);

                    tokio::select! {
                        _ = timeout.as_mut() =>{
                            if let Some(pc) = weak_peer.upgrade(){
                                result = pc.rtp_peer.write_rtcp(&[Box::new(PictureLossIndication{
                                    sender_ssrc: 0,
                                    media_ssrc,
                                })]).await.map_err(Into::into);
                            }else{
                                break;
                            }
                        }
                    }
                }
            });

            let sfu2 = sfu.clone();


            let peer = Arc::clone(&peer);
            tokio::spawn(async move {
                let rooms = sfu2.rooms.lock().await;

                rooms.get(&room_id).unwrap().lock().await.iter().for_each(
                    |(participant_id, participant_peer)| {
                        if peer.session_id != *participant_id {
                            let sfu2 = sfu2.clone();
                            let track = Arc::clone(&track);
                            // tokio::spawn(async move {
                            //     sfu2.connect_peer(Arc::clone(&track), participant_peer).await;
                            // });
                        }
                    },
                );
            });

            Box::pin(async {})
        }));

        Arc::clone(&o_peer)
    }

    fn on_pear_track() {}

    fn accept_offer(&mut self, session_id: String, sdp: RTCSessionDescription) {}

    async fn connect_peer(&self, track: Arc<TrackRemote>, dist: &Peer) {
        let local_track = Arc::new(TrackLocalStaticRTP::new(
            track.codec().capability,
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        let local_track2 = Arc::clone(&local_track);
        dist.rtp_peer.add_track(local_track).await.unwrap();

        // print!("send rtp to {} -> {}", &session_id3, &other_session);
        // Read RTP packets being sent to webrtc-rs
        while let Ok((rtp, _)) = track.read_rtp().await {
            if let Err(err) = local_track2.write_rtp(&rtp).await {
                if Error::ErrClosedPipe != err {
                    print!("output track write_rtp got error: {err} and break");
                    break;
                } else {
                    print!("output track write_rtp got error: {err}");
                }
            }
        }
    }
}

async fn candidates(
    State(app_state): State<AppState>,
    Json(candidates_req): Json<CandidatesRequest>,
) -> impl IntoResponse {
    if candidates_req.candidates.len() == 0 {}

    if candidates_req.session_id.len() == 0 {}

    let sessions = Arc::clone(&app_state.sessions).lock().await;
    let session = sessions.get(&candidates_req.session_id);

    if session.is_none() {
        return "";
    }

    let peer = app_state.sfu.clone().peers.lock().await.get(&candidates_req.session_id);

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
        .for_each(|c| block_on(peer.rtp_peer.add_ice_candidate(c.clone())).unwrap());

    "ok"
}

#[derive(Deserialize, Serialize)]
struct SdpRequest {
    offer: RTCSessionDescription,
    session_id: String,
    room_id: String,
}

async fn sdp(State(app_state): State<AppState>, Json(req): Json<SdpRequest>) -> impl IntoResponse {
    let peer = app_state.sfu.new_peer(req.session_id, req.room_id).await;

    // peer_connection.set_remote_description(req.offer).await.unwrap();
    // let answer = peer_connection.create_answer(None).await.unwrap();
    // let mut gather_complete = peer_connection.gathering_complete_promise().await;
    // peer_connection.set_local_description(answer.clone()).await.unwrap();
    // let _ = gather_complete.recv().await;

    //let peer = Arc::clone(&peer);
    let answer = RTCSessionDescription::default(); // TODO
    serde_json::to_string(&AnswerResponse {
        answer,
        session_id: peer.session_id.clone(),
    })
    .unwrap()
}

pub async fn create_peer() -> webrtc::error::Result<RTCPeerConnection> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec![
                "stun:stun.l.google.com:19302".to_owned(),
                "stun:stun.l.google.com:5349".to_owned(),
                "stun:stun1.l.google.com:3478".to_owned(),
                "stun:stun1.l.google.com:5349".to_owned(),
                "stun:stun2.l.google.com:19302".to_owned(),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    api.new_peer_connection(config).await
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

    let sfu = Arc::clone(&app_state.sfu);
    let sessions = Arc::clone(&app_state.sessions);

    ws.on_failed_upgrade(|error| {
        block_on(app_state.sessions.lock()).remove(&req.session_id);
    })
    .on_upgrade(|socket| async {
        let (mut sender, mut receiver) = socket.split();
        let socket_client: SocketClient = SocketClient::new((sender, receiver));

        app_state
            .sessions
            .lock()
            .await
            .insert(req.session_id.clone(), Arc::clone(&socket_client));

        handle_socket(
            sfu,
            req.session_id,
            Arc::downgrade(&socket_client),
        ).await;
    })
}

async fn process_message(
    sfu: Arc<SFU>,
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

async fn handle_socket(
    sfu: Arc<SFU>,
    session_id: String,
    socket_client: Weak<(SplitSink<WebSocket, Message>, SplitStream<WebSocket>)>,
) {
    let sfu = Arc::clone(&sfu);

    tokio::spawn(async move {
        loop {
            let socket_client = socket_client.upgrade();
            if socket_client.is_none() {
                break;
            }
            let socket_client = socket_client.unwrap();
            let msg = socket_client.1.next().await;

            if let Some(result) = msg {
                if result.is_ok() {
                    let msg = result.unwrap();
                    if process_message(Arc::clone(&sfu), msg, session_id.clone())
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

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, SocketClient>>>,
    sfu: SfuService,
}

pub async fn start_webrtc() -> Router {
    let state = AppState {
        sfu: SfuService(Arc::new(SFU {
            rooms: Mutex::new(HashMap::new()),
            peers: Default::default(),
        })),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/ws", any(ws))
        .route("/sdp", post(sdp))
        .route("/candidates", post(candidates))
        .layer(CorsLayer::permissive())
        .with_state(state);

    app
}
