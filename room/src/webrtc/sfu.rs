use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Weak};

use anyhow::{bail, Result};
use std::ops::{Deref};
use std::pin::Pin;
use log::{error, info, warn};
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
use webrtc::track::track_local::{TrackLocalWriter};
use webrtc::track::track_remote::TrackRemote;
use webrtc::Error;
use webrtc::Error::ErrNoRemoteDescription;

pub struct Participant {
    pub(crate) session_id: String,
    pub(crate) pc: RTCPeerConnection,
}


pub struct SFUInner {
    rooms: Mutex<HashMap<String, Arc<Mutex<HashMap<String, Arc<Participant>>>>>>,
    pub(crate) participants: Mutex<HashMap<String, Arc<Participant>>>,
    pub(crate) remote_tracks: Mutex<HashMap<String, Weak<TrackRemote>>>,
    candidates_buffers: Mutex<HashMap<String, Vec<RTCIceCandidateInit>>>,
    //session: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
    signalling: Box<dyn Signalling>
}

// Selective Forwarding unit
pub struct SFU(Arc<SFUInner>);

pub trait Signalling: Sync + Send {
    fn send_sdp(&self, string: String, sdp: RTCSessionDescription) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    fn send_ice_candidate(&self, session_id: String, candidate: Option<RTCIceCandidate>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

impl SFU {
    pub fn new(signalling: Box<dyn Signalling>) -> Self {
        SFU(Arc::new(SFUInner{
            signalling,
            participants: Default::default(),
            rooms: Default::default(),
            remote_tracks: Default::default(),
            candidates_buffers: Default::default(),
        }))
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
    async fn get_or_create_peer(&self, session_id: String, room_id: String) -> Result<Arc<Participant>> {
        let room = self.rooms.lock().await.entry(room_id.clone()).or_default().clone();
        let mut room_map = room.lock().await;

        let peer: Arc<Participant>;
        if room_map.contains_key(&session_id) {
            return match room_map.get(&session_id) {
                Some(peer) => Ok(peer.clone()),
                None => anyhow::bail!("Could not find peer {}", session_id)
            }
        }

        let pc = create_peer(session_id.clone()).await?;
        peer = Arc::new(Participant {
            session_id: session_id.clone(),
            pc: pc,
        });

        room_map.insert(peer.session_id.clone(), Arc::clone(&peer));
        let mut participants = self.participants.lock().await;
        if let Some(participant) = participants.get(&session_id) {
            _ = participant.pc.close().await;
        }
        participants.insert(session_id.clone(), Arc::clone(&peer));

        let this = self.clone();
        let session_id = session_id.clone();
        let this = this.clone();
        peer.pc.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
                let this = this.clone();
                let session_id = session_id.clone();
                Box::pin(async move {
                    match this.signalling.send_ice_candidate(session_id.clone(), c).await {
                        Ok(_) => {},
                        Err(e) => {
                            warn!(err:? = e, user:? = session_id.clone(); "Could not send ice candidate");
                        }
                    }
                })
            }));

        let this = self.clone();
        let room2 = Arc::clone(&room);
        let room_id2 = room_id.clone();
        let session_id = peer.session_id.clone();
        let w_peer = Arc::downgrade(&peer);
        Arc::clone(&peer).pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let room = Arc::clone(&room2);
            let this = this.clone();
            let room_id2 = room_id2.clone();
            let session_id = session_id.clone();
            let w_peer = w_peer.clone();
            Box::pin(async move {
                let Some(peer) = w_peer.upgrade() else {
                    return;
                };

                let room_id2 = room_id2.clone();
                match s {
                    RTCPeerConnectionState::Closed | RTCPeerConnectionState::Failed => {
                        match room.lock().await.remove(session_id.as_str()) {
                            None => warn!(user:? = session_id.clone(), room:? = room_id2.clone(); "Not found session_id in room"),
                            _ => {}
                        }
                        match this.participants.lock().await.remove(session_id.as_str()) {
                            None => warn!(user:? = session_id.clone(), room:? = room_id2.clone(); "Not found session_id in room"),
                            _ => {}
                        }

                        this.candidates_buffers.lock().await.remove(&session_id);
                        this.remote_tracks.lock().await.remove(&session_id);
                    }
                    RTCPeerConnectionState::Connected => {
                        this.on_connected(peer, room).await;
                    }
                    _ => {}
                }

                let peers = this.participants.lock().await.iter()
                    .map(|(_, p)| p.session_id.clone()).collect::<Vec<String>>();
                info!(
                        user:? = session_id,
                        room:? = room_id2.clone(),
                        peers:? = peers,
                        state:? = s;
                        "Peer state changed");
            })
        }));


        let session_id = peer.session_id.clone();
        let this = self.clone();
        let w_peer = Arc::downgrade(&peer);
        let room_id = room_id.clone();
        peer.pc.on_track(Box::new(move |track, _, _| {
            Box::pin({
                info!(user:? = session_id, room:? = room_id.clone(); "Peer on_track triggered");

                let this = this.clone();
                let w_peer = w_peer.clone();
                let room_id = room_id.clone();
                async move {
                    this.on_track(w_peer, room_id, track).await;
                }
            })
        }));

        Ok(Arc::clone(&peer))
    }

    async fn on_negotiation_needed(&self, peer: Arc<Participant>) -> Result<()> {
        let sdp = peer.pc.create_offer(None).await?;
        peer
            .pc
            .set_local_description(sdp.clone())
            .await?;

        self.signalling.send_sdp(peer.session_id.clone(), sdp).await
    }

    async fn on_track(&self, peer: Weak<Participant>, room_id: String, new_track: Arc<TrackRemote>) {
        let rooms = self.rooms.lock().await;
        let room = rooms.get(&room_id);
        if room.is_none() {
            warn!(room:? = room_id.clone(); "No room found");

            return;
        }
        let room = room.unwrap();

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
                            // println!("strong count {:?}", Arc::strong_count(&pc));

                            result = pc.pc.write_rtcp(&[Box::new(PictureLossIndication{
                                sender_ssrc: 0,
                                media_ssrc,
                            })]).await.map_err(Into::into);
                        } else {
                            break; // safe exit
                        }
                    }
                }
            }
            info!("Peer left");
        });

        let this = self.clone();


        if let Some(peer) = peer2.upgrade() {
            let session_id = peer.session_id.clone();
            this.remote_tracks.lock().await.insert(session_id.clone(), Arc::downgrade(&new_track));

            let participants = room.lock().await.clone().into_iter()
                .filter(move |(_, participant)| participant.session_id.clone() != session_id.clone())
                .filter(move |(_, participant)| participant.pc.connection_state().eq(&RTCPeerConnectionState::Connected));


            participants.for_each(|(_, participant)| {
                let new_track = Arc::clone(&new_track);
                let participant = Arc::clone(&participant);
                let this = this.clone();
                tokio::spawn(async move {
                    this.send_track_to_participant(new_track, participant).await;
                });
            });
        }
    }

    pub async fn accept_offer(&self, session_id: String, offer: RTCSessionDescription, room_id: String) -> Result<RTCSessionDescription> {
        let peer = self.get_or_create_peer(session_id.clone(), room_id).await?;
        peer.pc.set_remote_description(offer).await?;

        let _ = match self.candidates_buffers.lock().await.get_mut(&session_id) {
            Some(candidates) => {
                while let Some(candidate) = candidates.pop() {
                    peer.pc.add_ice_candidate(candidate).await?
                }

                Ok::<(), Error>(())
            }
            _ => Ok(())
        }?;

        let answer = peer.pc.create_answer(None).await?;

        let mut gather_complete = peer.pc.gathering_complete_promise().await;
        peer.pc.set_local_description(answer.clone()).await?;
        let _ = gather_complete.recv().await;

        Ok(answer)
    }

    pub(crate) async fn accept_answer(&self, session_id: String, answer: RTCSessionDescription, room_id: String) -> Result<()> {
        let Some(room) = self.rooms.lock().await.get(&room_id).cloned() else {
            bail!("room not found")
        };

        let Some(peer) = room.lock().await.get(&session_id).cloned() else {
            bail!("No peer found for this session")
        };

        peer.pc.set_remote_description(answer).await?;
        Ok(())
    }

    pub async fn accept_candidate(&self, session_id: String, room_id: String, candidate: RTCIceCandidateInit) -> Result<()> {
        let peer = self.get_or_create_peer(session_id.clone(), room_id).await?;

        match peer.pc.add_ice_candidate(candidate.clone()).await {
            Ok(_) => {
                let mut candidates = self.candidates_buffers.lock().await;
                let candidates = candidates.get_mut(&session_id);
                match candidates {
                    Some(candidates) => {
                        while let Some(cand) = candidates.pop() {
                            if let Err(e) = peer.pc.add_ice_candidate(cand.clone()).await {
                                // Если ошибка - возвращаем кандидат обратно и прерываем цикл
                                candidates.push(cand);
                                return Err(e.into());
                            }
                        }
                    }
                    _ => {}
                }

                Ok(())
            },
            Err(e) => match e {
                ErrNoRemoteDescription => {
                    self.candidates_buffers.lock().await.entry(session_id).or_insert_with(|| Vec::new()).push(candidate);
                    Ok(())
                },
                _ => Err(e.into()),
            },
        }
    }

    async fn send_track_to_participant(&self, track: Arc<TrackRemote>, dist: Arc<Participant>) {
        let dist_track = Arc::new(TrackLocalStaticRTP::new(
            track.codec().capability,
            track.id() + "-to-" + dist.session_id.as_str(),
            track.id(),
        ));

        let dist_track2 = Arc::clone(&dist_track);
        dist.pc.add_track(dist_track2).await.unwrap();

        let dist2 = Arc::clone(&dist);
        if let Err(e) = self.on_negotiation_needed(Arc::clone(&dist)).await {
            error!(user:? = dist2.session_id.clone(), err:? = e; "Failed await negotiation_needed");
            return;
        }

        // print!("send rtp to {} -> {}", &session_id3, &other_session);
        // Read RTP packets being sent to webrtc-rs

        let dist_track = Arc::downgrade(&dist_track);
        while let Ok((rtp, _)) = track.read_rtp().await {
            if let Some(dist_track) = dist_track.upgrade() {
                if let Err(err) = dist_track.write_rtp(&rtp).await {
                    if Error::ErrClosedPipe != err {
                        warn!(err:? = err; "output track write_rtp got error and break");
                        break;
                    } else {
                        warn!(err:? = err; "output track write_rtp got error");
                    }
                }
            } else {
                break
            }
        }

        info!("Track sending finished");
    }

    async fn on_connected(&self, new_peer: Arc<Participant>, room: Arc<Mutex<HashMap<String, Arc<Participant>>>>) {
        let session_id = new_peer.session_id.clone();

        // 1. Получаем список участников (без блокировки всей комнаты)
        let participants = {
            let room = room.lock().await;
            room.iter()
                .filter(|(_, p)| p.session_id != session_id && p.pc.connection_state() == RTCPeerConnectionState::Connected)
                .map(|(id, p)| (id.clone(), Arc::clone(p)))
                .collect::<Vec<_>>()
        };

        // 2. Обрабатываем каждого участника асинхронно
        let mut join_handles = Vec::new();

        let mut remote_tracks = self.remote_tracks.lock().await;

        let tracks = participants.iter().map(|(_, participant)|  {
            (participant.session_id.clone(), remote_tracks.get(&participant.session_id).and_then(|t| t.upgrade()))
        }).collect::<Vec<_>>();

        for (session_id, track) in tracks {
            let this = self.clone();
            let new_peer = new_peer.clone();

            match track {
                Some(track) => {
                    join_handles.push(tokio::spawn(async move {
                        // 4. Отправляем трек новому участнику
                        this.send_track_to_participant(track, new_peer).await;
                    }));
                },
                None => {remote_tracks.remove(&session_id);},
            };
        };
    }
}



pub async fn create_peer(session_id: String) -> webrtc::error::Result<RTCPeerConnection> {
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
                "stun:stun.sipnet.net:3478".to_owned(),
                "stun:stun.sipnet.ru:3478".to_owned(),
                "stun:stun.stunprotocol.org:3478".to_owned(),
            ],
            ..Default::default()
        }],
        peer_identity: session_id.clone(),
        ..Default::default()
    };
    api.new_peer_connection(config).await
}

