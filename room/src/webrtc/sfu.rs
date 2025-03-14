use std::collections::HashMap;
use std::future::Future;
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
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::ops::{ControlFlow, Deref};
use std::pin::Pin;
use futures::channel::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::sync::oneshot::channel;
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


pub struct Peer {
    pub session_id: String,
    pub(crate) rtp_peer: RTCPeerConnection,
    pub remote_tracks: Vec<TrackRemote>,
    //local_tracks: Vec<Box<dyn TrackLocal>>,
}


pub struct SFUInner {
    rooms: Mutex<HashMap<String, Arc<Mutex<HashMap<String, Arc<Peer>>>>>>,
    pub(crate) peers: Mutex<HashMap<String, Arc<Peer>>>,
    //session: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
    signalling: Box<dyn Signalling>
}

// Selective Forwarding unit
pub struct SFU(Arc<(SFUInner)>);

pub trait Signalling: Sync + Send {
    fn send_sdp(&self, sdp: RTCSessionDescription) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl SFU {
    pub fn new(
        signalling: Box<dyn Signalling>,
        //session: Arc<Mutex<HashMap<String, Arc<()>>>>
    ) -> Self {
        let inner = Arc::new(SFUInner{
            signalling,
            peers: Default::default(),
            rooms: Default::default(),
        });
        let sfu: Self = Self(inner);

        //channel();

        sfu
    }
}

impl Clone for SFU {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for SFU {
    type Target = SFUInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SFU {
    pub async fn new_peer(&self, session_id: String, room_id: String) -> Arc<Peer> {
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
        //let session = Arc::clone(&self.session);

        let this = self.clone();
        peer.rtp_peer.on_negotiation_needed(Box::new(move || {
            Box::pin({
                let peer2 = Arc::clone(&peer2);

                println!("Peer Connection Negotiation needed: {:?}", peer2.rtp_peer);

          //      let session = Arc::clone(&session);
                let this = this.clone();
                async move {
                    let sdp = peer2.rtp_peer.create_offer(None).await;
                    if sdp.is_err() {
                        println!("Peer Connection negotiation failed, error: {:?}", sdp.unwrap_err());
                        return;
                    }
                    let sdp = sdp.unwrap();
                    peer2
                        .rtp_peer
                        .set_local_description(sdp.clone())
                        .await
                        .unwrap();

                    // let binding = session.lock().await;
                    // let socket_client = binding.get(peer2.session_id.as_str());
                    // if socket_client.is_none() {
                    //     println!("Peer Connection hasnt socket client");
                    //     return
                    // }
                    // let socket_client = socket_client.unwrap();
                    // let json = serde_json::to_string(&sdp).unwrap();
                    //
                    // let result = socket_client.0.lock().await.send(Message::from(json)).await;
                    // if result.is_err() {
                    //     println!("Peer Connection negotiation failed, error: {:?}", result.unwrap_err());
                    //     return
                    // }

                    this.signalling.send_sdp(sdp).await;
                }
            })
        }));


        // let peer2 = Arc::clone(&peer);
        let peer2 = Arc::clone(&peer);
        let sfu = self.clone();

        block_on(sfu.peers.lock()).insert(peer2.session_id.clone(), Arc::clone(&peer2));

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

            let room_id = room_id.clone();
            // peer2.local_tracks.insert(session_id.clone(), track.clone());
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
            // broadcast all participants
            let peer = Arc::clone(&peer);
            tokio::spawn(async move {
                let sfu3 = sfu2.clone();
                let rooms = sfu2.rooms.lock().await;

                let room = rooms.get(&room_id);
                if room.is_none() {
                    println!("No room found for this session");
                    return;
                }
                let room = room.unwrap();

                let room = Arc::clone(&room);
                let participants = room.lock().await.clone().into_iter()
                    .filter(|(_, participant)| participant.session_id != peer.session_id);

                participants.for_each(|(_, participant)| {
                    let sfu3 = sfu3.clone();
                    let track = Arc::clone(&track);
                    tokio::spawn(async move {
                        sfu3.send_remote_track(track, &participant).await;
                    });
                });
            });

            Box::pin(async {})
        }));

        Arc::clone(&o_peer)
    }

    fn on_pear_track() {}

    //fn accept_offer(&mut self, session_id: String, sdp: RTCSessionDescription) {}

    async fn send_remote_track(&self, track: Arc<TrackRemote>, dist: &Peer) {
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

