use std::any::Any;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use webrtc::rtp_transceiver::rtp_codec::RTPCodecType;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::{peer_connection, Error};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::str::FromStr;
use axum::{Json, Router,
           extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
};
use axum::body::Bytes;
use axum::extract::{ConnectInfo, Query, State};
use axum::response::IntoResponse;
use axum::routing::{any, get, post};
use base64::Engine;
use lazy_static::lazy_static;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use base64::prelude::BASE64_STANDARD;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use webrtc::peer_connection::RTCPeerConnection;
use crate::{create_user, root, CreateUser, User};
use tower_http::cors::CorsLayer;
use rand::{random, Rng};
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::track::track_remote::TrackRemote;

lazy_static! {
    static ref SDP_CHAN_TX_MUTEX: Arc<Mutex<Option<mpsc::Sender<String>>>> =
        Arc::new(Mutex::new(None));
}


async fn remote_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // A HTTP handler that processes a SessionDescription given to us from the other WebRTC-rs or Pion process
        (&Method::POST, "/sdp") => {
            //println!("remote_handler receive from /sdp");
            let sdp_str = match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?)
            {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("{}", err),
            };

            {
                let sdp_chan_tx = SDP_CHAN_TX_MUTEX.lock().await;
                if let Some(tx) = &*sdp_chan_tx {
                    let _ = tx.send(sdp_str).await;
                }
            }

            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::OK;
            Ok(response)
        }
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub fn decode(s: &str) -> Result<String> {
    let mut b = vec![];
    BASE64_STANDARD.decode_vec(s, &mut b)?;

    //if COMPRESS {
    //    b = unzip(b)
    //}

    let s = String::from_utf8(b)?;
    Ok(s)
}

/// encode encodes the input in base64
/// It can optionally zip the input before encoding
pub fn encode(b: &str) -> String {
    //if COMPRESS {
    //    b = zip(b)
    //}

    BASE64_STANDARD.encode(b)
}


fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::thread_rng();

    let random_string: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    random_string
}

#[derive(Deserialize, Serialize)]
struct AnswerResponse {
    answer: RTCSessionDescription,
    session_id: String,
}

#[derive(Deserialize, Serialize)]
struct CandidatesRequest {
    candidates: Vec<RTCIceCandidateInit>,
    session_id: String,
}


async fn candidates(
    State(app_state): State<AppState>,
    Json(candidates_req): Json<CandidatesRequest>,
) -> impl IntoResponse {
    if candidates_req.candidates.len() == 0 {

    }

    if candidates_req.session_id.len() == 0 {

    }

    let peer_connection = app_state.peers.lock().await.get(&candidates_req.session_id).unwrap().clone();
    candidates_req.candidates.iter().for_each(|c| {
        block_on(peer_connection.add_ice_candidate(c.clone())).unwrap()
    });

    "ok"
}

#[derive(Deserialize, Serialize)]
struct SdpRequest {
    offer: RTCSessionDescription,
    session_id: String,
}

async fn sdp(
    State(app_state): State<AppState>,
    Json(req): Json<SdpRequest>,
) -> impl IntoResponse {

    let mut peer_connection = Arc::new(block_on(create_peer()).unwrap());
    let session_id = req.session_id; //generate_random_string(5);

    if block_on(app_state.peers.lock()).contains_key(&session_id) {
       peer_connection = block_on(app_state.peers.lock()).get(&session_id).unwrap().clone();
    } else {
        peer_connection.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            println!("Peer Connection on ice candidate: {:?}", c);

            Box::pin(async {})
        }));


        let peers = Arc::clone(&app_state.peers);
        let conns = Arc::clone(&app_state.conns);
        let session_id3 = session_id.clone();
        let peer_connection2 = Arc::clone(&peer_connection);
        peer_connection.on_negotiation_needed(Box::new(move || {
            let conns1 = block_on(conns.lock());
            let other_conn = conns1.get(&session_id3).unwrap();
            print!("on_negotiation_needed");

            let sdp = block_on(peer_connection2.create_offer(None)).unwrap();
            block_on(peer_connection2.set_local_description(sdp.clone())).unwrap();

            block_on(block_on(other_conn.lock()).send(Message::from(serde_json::to_string(&sdp).unwrap()))).unwrap();

            Box::pin(async {})
        }));

        let session_id2 = session_id.clone();
        peer_connection.on_track(Box::new(move |track, r, t| {

            block_on(app_state.tracks.lock()).insert(session_id2.clone(), track.clone());

            let media_ssrc = track.ssrc();


            let mut other_pc: Option<Arc<RTCPeerConnection>> = None;

            let mut other_session: Option<String> = None;
            block_on(peers.lock()).iter().for_each(|(sid, pc)| {
                if session_id2 != *sid {
                    other_pc = Some(pc.clone());
                    other_session = Some(sid.clone());
                }
            });


            if other_pc.is_some() {
                let other_pc = Arc::downgrade(&other_pc.unwrap());

                tokio::spawn(async move {
                    let mut result = Result::<usize>::Ok(0);
                    while result.is_ok() {
                        let timeout = tokio::time::sleep(Duration::from_secs(3));
                        tokio::pin!(timeout);

                        tokio::select! {
                    _ = timeout.as_mut() =>{
                        if let Some(pc) = other_pc.upgrade(){
                            result = pc.write_rtcp(&[Box::new(PictureLossIndication{
                                sender_ssrc: 0,
                                media_ssrc,
                            })]).await.map_err(Into::into);
                        }else{
                            break;
                        }
                    }
                };
                    }
                });

                let conns = app_state.conns.clone();
                let session_id3 = session_id2.clone();
                let peers2 = Arc::clone(&peers);
                tokio::spawn(async move {
                    // Create Track that we send video back to browser on
                    let local_track = Arc::new(TrackLocalStaticRTP::new(
                        track.codec().capability,
                        "video".to_owned(),
                        "webrtc-rs".to_owned(),
                    ));


                    let other_session = other_session.unwrap();
                    let peers3 = peers2.lock().await;
                    let other_peer = peers3.get(&other_session).unwrap();
                    other_peer.add_track(local_track.clone()).await.unwrap();
                    drop(peers3);

                    // Read RTP packets being sent to webrtc-rs
                    while let Ok((rtp, _)) = track.read_rtp().await {
                        if let Err(err) = local_track.write_rtp(&rtp).await {
                            if Error::ErrClosedPipe != err {
                                print!("output track write_rtp got error: {err} and break");
                                break;
                            } else {
                                print!("output track write_rtp got error: {err}");
                            }
                        }
                    }
                });

            }

            Box::pin(async {})
        }));

        let session_id2 = session_id.clone();
        let peers3 =  Arc::clone(&app_state.peers);
        let peers4 =  Arc::clone(&app_state.peers);
        peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {s}");
            match s {
                RTCPeerConnectionState::Closed | RTCPeerConnectionState::Disconnected | RTCPeerConnectionState::Failed => {
                    block_on(peers4.lock()).remove(&session_id2);
                },
                _ => {},
            }
            Box::pin(async {})
        }));

        peers3.lock().await.insert(session_id.clone(), Arc::clone(&peer_connection));
    }

    peer_connection.set_remote_description(req.offer).await.unwrap();

    let answer = peer_connection.create_answer(None).await.unwrap();

    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    peer_connection.set_local_description(answer.clone()).await.unwrap();

    let _ = gather_complete.recv().await;

    serde_json::to_string(&AnswerResponse{answer, session_id: session_id}).unwrap()
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
    ws.on_upgrade(move |socket| {
        let socket = Arc::new(Mutex::new(socket));
        block_on(app_state.conns.lock()).insert(req.session_id.clone(), Arc::clone(&socket));
        handle_socket(socket, req.session_id, Arc::clone(&app_state.conns))
    })
}

fn process_message(msg: Message) -> ControlFlow<(), ()> {
    let who = "1";
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

async fn handle_socket(mut socket: Arc<Mutex<WebSocket>>, session_id: String, conns: Arc<Mutex<HashMap<String, Arc<Mutex<WebSocket>>>>>) {
    let who = session_id;
    if socket.lock().await
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    if let Some(msg) = socket.lock().await.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg).is_break() {
                return;
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }

    for i in 1..5 {
        if socket.lock().await
            .send(Message::Text(format!("Hi {i} times!").into()))
            .await
            .is_err()
        {
            println!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[derive(Clone, Default)]
struct AppState {
    peers: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    tracks: Arc<Mutex<HashMap<String, Arc<TrackRemote>>>>,
    local_tracks: Arc<Mutex<HashMap<String, Arc<TrackRemote>>>>,
    conns: Arc<Mutex<HashMap<String, Arc<Mutex<WebSocket>>>>>,
}


pub async fn start_webrtc() -> Router {
    let (sdp_chan_tx, sdp_chan_rx) = mpsc::channel::<String>(1);
    {
        let mut tx = SDP_CHAN_TX_MUTEX.lock().await;
        *tx = Some(sdp_chan_tx);
    }


    let state = AppState::default();

    println!("111");
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/ws", any(ws))
        .route("/sdp", post(sdp))
        .route("/candidates", post(candidates))
        .layer(CorsLayer::permissive())
        .with_state(state)
        // `POST /users` goes to `create_user`
        // .route("/candidates", post(candidates))
        ;

    return app;
}
pub async fn start_webrtc1(sdp_chan_tx: Sender<String>, mut sdp_chan_rx: Receiver<String>) -> Result<()>  {

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(create_peer().await?);



    // Allow us to receive 1 video track
    peer_connection
        .add_transceiver_from_kind(RTPCodecType::Video, None)
        .await?;


    let (local_track_chan_tx, mut local_track_chan_rx) =
        tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(1);

    let local_track_chan_tx = Arc::new(local_track_chan_tx);
    // Set a handler for when a new remote track starts, this handler copies inbound RTP packets,
    // replaces the SSRC and sends them back
    let pc = Arc::downgrade(&peer_connection);
    peer_connection.on_track(Box::new(move |track, _, _| {
        // Send a PLI on an interval so that the publisher is pushing a keyframe every rtcpPLIInterval
        // This is a temporary fix until we implement incoming RTCP events, then we would push a PLI only when a viewer requests it
        let media_ssrc = track.ssrc();
        let pc2 = pc.clone();
        tokio::spawn(async move {
            let mut result = Result::<usize>::Ok(0);
            while result.is_ok() {
                let timeout = tokio::time::sleep(Duration::from_secs(3));
                tokio::pin!(timeout);

                tokio::select! {
                    _ = timeout.as_mut() =>{
                        if let Some(pc) = pc2.upgrade(){
                            result = pc.write_rtcp(&[Box::new(PictureLossIndication{
                                sender_ssrc: 0,
                                media_ssrc,
                            })]).await.map_err(Into::into);
                        }else{
                            break;
                        }
                    }
                };
            }
        });

        let local_track_chan_tx2 = Arc::clone(&local_track_chan_tx);
        tokio::spawn(async move {
            // Create Track that we send video back to browser on
            let local_track = Arc::new(TrackLocalStaticRTP::new(
                track.codec().capability,
                "video".to_owned(),
                "webrtc-rs".to_owned(),
            ));
            let _ = local_track_chan_tx2.send(Arc::clone(&local_track)).await;

            // Read RTP packets being sent to webrtc-rs
            while let Ok((rtp, _)) = track.read_rtp().await {
                if let Err(err) = local_track.write_rtp(&rtp).await {
                    if Error::ErrClosedPipe != err {
                        print!("output track write_rtp got error: {err} and break");
                        break;
                    } else {
                        print!("output track write_rtp got error: {err}");
                    }
                }
            }
        });

        Box::pin(async {})
    }));

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        Box::pin(async {})
    }));

    // Set the remote SessionDescription
//    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let r = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        let b64 = encode(&json_str);
        println!("{b64}");
    } else {
        println!("generate local_description failed!");
    }

    if let Some(local_track) = local_track_chan_rx.recv().await {
        loop {
            println!("\nCurl an base64 SDP to start sendonly peer connection");

            let line = sdp_chan_rx.recv().await.unwrap();
            let desc_data = decode(line.as_str())?.trim().to_string();
            let recv_only_offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

            // Create a MediaEngine object to configure the supported codec
            let mut m = MediaEngine::default();

            m.register_default_codecs()?;

            // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
            // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
            // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
            // for each PeerConnection.
            let mut registry = Registry::new();

            // Use the default set of Interceptors
            registry = register_default_interceptors(registry, &mut m)?;

            // Create the API object with the MediaEngine
            let api = APIBuilder::new()
                .with_media_engine(m)
                .with_interceptor_registry(registry)
                .build();

            // Prepare the configuration
            let config = RTCConfiguration {
                ice_servers: vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                }],
                ..Default::default()
            };

            // Create a new RTCPeerConnection
            let peer_connection = Arc::new(api.new_peer_connection(config).await?);

            let rtp_sender = peer_connection
                .add_track(Arc::clone(&local_track) as Arc<dyn TrackLocal + Send + Sync>)
                .await?;

            // Read incoming RTCP packets
            // Before these packets are returned they are processed by interceptors. For things
            // like NACK this needs to be called.
            tokio::spawn(async move {
                let mut rtcp_buf = vec![0u8; 1500];
                while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
                Result::<()>::Ok(())
            });

            // Set the handler for Peer connection state
            // This will notify you when the peer has connected/disconnected
            peer_connection.on_peer_connection_state_change(Box::new(
                move |s: RTCPeerConnectionState| {
                    println!("Peer Connection State has changed: {s}");
                    Box::pin(async {})
                },
            ));

            // Set the remote SessionDescription
            peer_connection
                .set_remote_description(recv_only_offer)
                .await?;

            // Create an answer
            let answer = peer_connection.create_answer(None).await?;

            // Create channel that is blocked until ICE Gathering is complete
            let mut gather_complete = peer_connection.gathering_complete_promise().await;

            // Sets the LocalDescription, and starts our UDP listeners
            peer_connection.set_local_description(answer).await?;

            // Block until ICE Gathering is complete, disabling trickle ICE
            // we do this because we only can exchange one signaling message
            // in a production application you should exchange ICE Candidates via OnICECandidate
            let _ = gather_complete.recv().await;

            if let Some(local_desc) = peer_connection.local_description().await {
                let json_str = serde_json::to_string(&local_desc)?;
                let b64 = encode(&json_str);
                println!("{b64}");
            } else {
                println!("generate local_description failed!");
            }
        }
    }

    Ok(())
}