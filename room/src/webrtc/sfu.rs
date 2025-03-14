use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Weak};

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
use std::ops::{Deref};
use std::pin::Pin;
use tokio::sync::Mutex;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
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
    fn send_sdp(&self, string: String, sdp: RTCSessionDescription) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl SFU {
    pub fn new(signalling: Box<dyn Signalling>) -> Self {
        let inner = Arc::new(SFUInner{
            signalling,
            peers: Default::default(),
            rooms: Default::default(),
        });
        Self(inner)
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
    async fn get_peer(&self, session_id: String) -> Option<Arc<Peer>> {
        self.peers.lock().await.get(&session_id).map(|peer| peer.clone())
    }

    async fn new_peer(&self, session_id: String, room_id: String) -> Arc<Peer> {
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

                    this.signalling.send_sdp(peer2.session_id.clone(), sdp).await;
                }
            })
        }));


        // let peer2 = Arc::clone(&peer);
        let peer2 = Arc::clone(&peer);
        let sfu = self.clone();

        sfu.peers.lock().await.insert(peer2.session_id.clone(), Arc::clone(&peer2));

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


        let this = self.clone();
        let room_id = room_id.clone();

        let o_peer = Arc::clone(&peer);
        let peer = Arc::clone(&peer);
        Arc::clone(&peer).rtp_peer.on_track(Box::new(move |track, _, _| {

            let this = this.clone(); // TODO
            let peer = Arc::clone(&peer);
            let room_id = room_id.clone();
            Box::pin(async move {
                this.on_track(Arc::downgrade(&peer), room_id, track).await;
            })
        }));

        Arc::clone(&o_peer)
    }

    async fn on_track(&self, peer: Weak<Peer>, room_id: String, new_track: Arc<TrackRemote>) {
        let rooms = self.rooms.lock().await;
        let room = rooms.get(&room_id);
        if room.is_none() {
            println!("No room found for this session");
            return;
        }
        let room = room.unwrap();

        let room_id = room_id.clone();
        let media_ssrc = new_track.ssrc();
        let peer2 = peer.clone();
        tokio::spawn(async move {
            let mut result = Result::<usize>::Ok(0);
            while result.is_ok() {
                let timeout = tokio::time::sleep(Duration::from_secs(3));
                tokio::pin!(timeout);

                tokio::select! {
                    _ = timeout.as_mut() =>{
                        if let Some(pc) = peer.upgrade(){
                            result = pc.rtp_peer.write_rtcp(&[Box::new(PictureLossIndication{
                                sender_ssrc: 0,
                                media_ssrc,
                            })]).await.map_err(Into::into);
                        } else {
                            break; // safe exit
                        }
                    }
                }
            }
        });

        let this = self.clone();

        let p = peer2.clone().upgrade();
        if p.is_none() {
            return;
        }

        let session_id = p.unwrap().session_id.clone();

        let participants = room.lock().await.clone().into_iter()
            .filter(move |(_, participant)| participant.session_id != session_id);

        tokio::spawn(async move {
            participants.for_each(|(_, participant)| {
                let new_track = Arc::clone(&new_track);
                let participant = Arc::clone(&participant);
                let this = this.clone();
                tokio::spawn(async move {
                    this.send_track_to_participant(new_track, participant).await;
                });
            });
        });
    }

    pub async fn accept_offer(&self, session_id: String, offer: RTCSessionDescription, room_id: String) -> RTCSessionDescription {
        let peer = self.new_peer(session_id, room_id).await;
        peer.rtp_peer.set_remote_description(offer).await.unwrap();
        let answer = peer.rtp_peer.create_answer(None).await.unwrap();

        let mut gather_complete = peer.rtp_peer.gathering_complete_promise().await;
        peer.rtp_peer.set_local_description(answer.clone()).await.unwrap();
        let _ = gather_complete.recv().await;

        answer
    }

    pub(crate) async fn accept_answer(&self, session_id: String, answer: RTCSessionDescription, room_id: String) {
        let peer = self.get_peer(session_id).await;
        if peer.is_none() {
            println!("No peer found for this session"); // TODO error
            return;
        }

        peer.unwrap().rtp_peer.set_remote_description(answer).await.unwrap();
    }

    pub async fn accept_candidate(&self, session_id: String, candidate: RTCIceCandidateInit) -> () {
        let peer = self.get_peer(session_id).await;
        if peer.is_none() {
            println!("No peer found for this session"); // TODO error
            return;
        }

        peer.unwrap().rtp_peer.add_ice_candidate(candidate).await; // TODO error handling
    }

    async fn send_track_to_participant(&self, track: Arc<TrackRemote>, dist: Arc<Peer>) {
        let dist_track = Arc::new(TrackLocalStaticRTP::new(
            track.codec().capability,
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        let dist_track2 = Arc::clone(&dist_track);
        dist.rtp_peer.add_track(dist_track2).await.unwrap();

        // print!("send rtp to {} -> {}", &session_id3, &other_session);
        // Read RTP packets being sent to webrtc-rs
        while let Ok((rtp, _)) = track.read_rtp().await {
            if let Err(err) = dist_track.write_rtp(&rtp).await {
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

