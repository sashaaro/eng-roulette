
use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidateInit};

#[derive(Deserialize, Serialize)]
pub struct AnswerResponse {
    pub answer: RTCSessionDescription,
    pub session_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct CandidatesRequest {
    pub candidates: Vec<RTCIceCandidateInit>,
    pub session_id: String,
}