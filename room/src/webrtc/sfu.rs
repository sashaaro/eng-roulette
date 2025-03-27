use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Weak};

use anyhow::{bail, Result};
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
use webrtc::track::track_local::{TrackLocalWriter};
use webrtc::track::track_remote::TrackRemote;
use webrtc::Error;
use tokio_util::sync::CancellationToken;
use webrtc::Error::ErrNoRemoteDescription;

pub struct Peer {
    pub(crate) session_id: String,
    pub(crate) rtp_peer: RTCPeerConnection,
    pub(crate) cancel: CancellationToken,
}


pub struct SFUInner {
    rooms: Mutex<HashMap<String, Arc<Mutex<HashMap<String, Arc<Peer>>>>>>,
    pub(crate) peers: Mutex<HashMap<String, Arc<Peer>>>,
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
            peers: Default::default(),
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
    async fn get_peer(&self, session_id: String) -> Option<Arc<Peer>> {
        self.peers.lock().await.get(&session_id).map(|peer| peer.clone())
    }

    async fn get_or_create_room(&self, room_id: String) -> Arc<Mutex<HashMap<String, Arc<Peer>>>> {
        let mut rooms = self.rooms.lock().await;
        if !rooms.contains_key(&room_id) {
            rooms.insert(room_id.clone(), Default::default());
        }
        rooms.get(&room_id).unwrap().clone()
    }

    async fn get_or_create_peer(&self, session_id: String, room_id: String) -> Result<Arc<Peer>> {
        let room = self.get_or_create_room(room_id.clone()).await;
        let mut room_map = room.lock().await;

        let mut peer: Arc<Peer>;
        if room_map.contains_key(&session_id) {
            return match room_map.get(&session_id) {
                Some(peer) => Ok(peer.clone()),
                None => anyhow::bail!("Could not find peer {}", session_id)
            }
        }

        peer = Arc::new(Peer {
            session_id: session_id.clone(),
            rtp_peer: create_peer(session_id.clone()).await?,
            cancel: CancellationToken::new(),
        });

        room_map.insert(peer.session_id.clone(), Arc::clone(&peer));
        drop(room_map);

        let this = self.clone();
        let peer2 = Arc::clone(&peer);
        peer.rtp_peer
            .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
                let peer2 = Arc::clone(&peer2);
                let this = this.clone();

                Box::pin(async move {
                    match this.signalling.send_ice_candidate(peer2.session_id.clone(), c).await {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Error sending ice candidate: {:?}", e);
                        }
                    }
                })
            }));

        // let peer2 = Arc::clone(&peer);

        // let this = self.clone();
        // peer.rtp_peer.on_negotiation_needed(Box::new(move || {
        //     Box::pin({
        //         let peer2 = Arc::clone(&peer2);
        //         let peer3 = Arc::clone(&peer2);
        //         let this = this.clone();
        //
        //         async move {
        //             println!("Peer Connection {:?} Negotiation needed. Signaling state {:?}", peer2.session_id, peer2.rtp_peer.signaling_state());
        //             if peer2.rtp_peer.signaling_state() != RTCSignalingState::Stable {
        //                 return
        //             }
        //
        //             match this.on_negotiation_needed(peer2).await {
        //                 Err(e) => {
        //                     println!("negotiation needed error: {:?} {:?}", e, peer3.session_id);
        //                 },
        //                 _ => {}
        //             };
        //         }
        //     })
        // }));


        // let peer2 = Arc::clone(&peer);
        let peer2 = Arc::clone(&peer);
        self.clone().peers.lock().await.insert(peer2.session_id.clone(), Arc::clone(&peer2));

        let this = self.clone();
        let room2 = Arc::clone(&room);

        let cancel_token2 = peer.cancel.clone();
        Arc::clone(&peer).rtp_peer.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let peer2 = Arc::clone(&peer2);
            let room = Arc::clone(&room2);
            let this = this.clone();
            Box::pin(async move {
                println!("Peer {:?} Connection State has changed: {:?}", peer2.session_id, s);
                match s {
                    RTCPeerConnectionState::Closed
                    //| RTCPeerConnectionState::Disconnected
                    | RTCPeerConnectionState::Failed => {
                        match room.lock().await.remove(peer2.session_id.as_str()) {
                            None => println!("not found session_id in room"),
                            _ => {}
                        }
                        match this.peers.lock().await.remove(peer2.session_id.as_str()) {
                            None => println!("not found session_id in peers"),
                            _ => {}
                        }

                        this.candidates_buffers.lock().await.remove(&peer2.session_id);

                        let mut remote_tracks = this.remote_tracks.lock().await;
                        remote_tracks.remove(&peer2.session_id);

                        //cancel_token2.cancel();

                        println!("peers in room {:?}", room.lock().await.len());
                        println!("peers {:?}", this.peers.lock().await.len());
                        println!("tracks {:?}", remote_tracks.len());
                    }
                    RTCPeerConnectionState::Connected => {
                    }
                    _ => {}
                }
            })
        }));


        let this = self.clone();
        let room_id = room_id.clone();

        let o_peer = Arc::clone(&peer);
        let peer = Arc::clone(&peer);
        let room2 = Arc::clone(&room);
        Arc::clone(&peer).rtp_peer.on_track(Box::new(move |track, _, _| {
            let this = this.clone(); // TODO
            let peer = Arc::clone(&peer);
            let room_id = room_id.clone();
            let room = Arc::clone(&room2);

            Box::pin(async move {
                this.on_track(Arc::downgrade(&peer), room_id, track).await;

                this.on_connected(Arc::clone(&peer), room).await;
            })
        }));

        Ok(Arc::clone(&o_peer))
    }

    async fn on_negotiation_needed(&self, peer: Arc<Peer>) -> Result<()> {
        let sdp = peer.rtp_peer.create_offer(None).await?;
        peer
            .rtp_peer
            .set_local_description(sdp.clone())
            .await?;

        self.signalling.send_sdp(peer.session_id.clone(), sdp).await
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

        let p = p.unwrap();
        let session_id = p.session_id.clone();
        this.remote_tracks.lock().await.insert(session_id.clone(), Arc::downgrade(&new_track));

        let participants = room.lock().await.clone().into_iter()
            .filter(move |(_, participant)| participant.session_id.clone() != session_id.clone());

        tokio::spawn(async move {

            participants.for_each(|(_, participant)| {
                let new_track = Arc::clone(&new_track);
                let participant = Arc::clone(&participant);
                let participant2 = Arc::clone(&participant);
                let this = this.clone();
                let p = Arc::clone(&p);
                tokio::spawn(async move {
                    // TODO pass cancel signal after peer exit

                    let senders = p.rtp_peer.get_senders().await;


                    this.send_track_to_participant(new_track, participant).await;

                    if !senders.is_empty() {
                        let sender = senders.get(0).unwrap();
                        match participant2.rtp_peer.remove_track(sender).await {
                            Ok(_) => {}
                            Err(e) => {
                                println!("remove track error {:?}", e)
                            }
                        }
                    }
                });
            });
        });
    }

    pub async fn accept_offer(&self, session_id: String, offer: RTCSessionDescription, room_id: String) -> Result<RTCSessionDescription> {
        let peer = self.get_or_create_peer(session_id.clone(), room_id).await?;
        peer.rtp_peer.set_remote_description(offer).await?;

        let _ = match self.candidates_buffers.lock().await.get_mut(&session_id) {
            Some(candidates) => {
                while let Some(candidate) = candidates.pop() {
                    peer.rtp_peer.add_ice_candidate(candidate).await?
                }

                Ok::<(), Error>(())
            }
            _ => Ok(())
        }?;

        let answer = peer.rtp_peer.create_answer(None).await?;

        let mut gather_complete = peer.rtp_peer.gathering_complete_promise().await;
        peer.rtp_peer.set_local_description(answer.clone()).await?;
        let _ = gather_complete.recv().await;

        Ok(answer)
    }

    pub(crate) async fn accept_answer(&self, session_id: String, answer: RTCSessionDescription, room_id: String) -> Result<()> {
        let peer = self.get_peer(session_id).await;
        if peer.is_none() {
            return bail!("No peer found for this session");
        }

        peer.unwrap().rtp_peer.set_remote_description(answer).await?;
        Ok(())
    }

    pub async fn accept_candidate(&self, session_id: String, room_id: String, candidate: RTCIceCandidateInit) -> Result<()> {
        let peer = self.get_or_create_peer(session_id.clone(), room_id).await?;

        match peer.rtp_peer.add_ice_candidate(candidate.clone()).await {
            Ok(_) => {
                let mut candidates = self.candidates_buffers.lock();
                let mut candidates = candidates.await;
                let candidates = candidates.get_mut(&session_id);
                match candidates {
                    Some(candidates) => {
                        while let Some(cand) = candidates.pop() {
                            if let Err(e) = peer.rtp_peer.add_ice_candidate(cand.clone()).await {
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

    async fn send_track_to_participant(&self, track: Arc<TrackRemote>, dist: Arc<Peer>) {
        let dist_track = Arc::new(TrackLocalStaticRTP::new(
            track.codec().capability,
            track.id() + "-to-" + dist.session_id.as_str(),
            track.id(),
        ));

        let dist_track2 = Arc::clone(&dist_track);
        let dist_track3 = Arc::clone(&dist_track);
        dist.rtp_peer.add_track(dist_track2).await.unwrap();


        let dist2 = Arc::clone(&dist);
        match self.on_negotiation_needed(Arc::clone(&dist)).await {
            Err(e) => {
                println!("negotiation needed error: {:?}", dist2.session_id.clone());
                return;
            },
            _ => {}
        };

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

    async fn on_connected(&self, new_peer: Arc<Peer>, room: Arc<Mutex<HashMap<String, Arc<Peer>>>>) {

        let session_id = new_peer.session_id.clone();

        // 1. Получаем список участников (без блокировки всей комнаты)
        let participants = {
            let room = room.lock().await;
            room.iter()
                .filter(|(_, p)| p.session_id != session_id)
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
            ],
            ..Default::default()
        }],
        peer_identity: session_id.clone(),
        ..Default::default()
    };
    api.new_peer_connection(config).await
}

