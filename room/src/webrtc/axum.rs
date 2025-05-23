use crate::extract::jwt::{Jwt, SecretKey};
use crate::webrtc::sfu::{Sfu, Signalling};
use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{FromRef, State, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, post};
use axum::{Json, Router};
use futures::executor::block_on;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use http::StatusCode;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::DecodingKey;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub(crate) type SocketClient = (
    Mutex<SplitSink<WebSocket, Message>>,
    Mutex<SplitStream<WebSocket>>,
);

#[derive(Clone)]
pub struct WebrtcState {
    pub(crate) sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
    pub(crate) sfu: Sfu,
    pub secret_key: SecretKey,
}

impl FromRef<WebrtcState> for SecretKey {
    fn from_ref(app_state: &WebrtcState) -> SecretKey {
        app_state.secret_key
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "playground")]
pub enum SignalingResponse {
    #[serde(rename = "sdp")]
    Sdp(Box<RTCSessionDescription>),

    #[serde(rename = "candidate")]
    Candidate(Option<RTCIceCandidate>),
}

struct WebsocketSignalling {
    sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
}

impl WebsocketSignalling {
    fn new(sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>) -> Self {
        Self { sessions }
    }
}

#[derive(Error, Debug)]
pub enum SfuError {
    #[error("Session not found")]
    SessionNotFound,
}

impl Signalling for WebsocketSignalling {
    fn send_sdp(
        &self,
        session_id: String,
        sdp: RTCSessionDescription,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id);
            if session.is_none() {
                Err(SfuError::SessionNotFound.into())
            } else {
                let session = session.unwrap();
                let playground = serde_json::to_string(&SignalingResponse::Sdp(Box::new(sdp)))?;
                session
                    .0
                    .lock()
                    .await
                    .send(Message::from(playground))
                    .await?;
                Ok(())
            }
        })
    }

    fn send_ice_candidate(
        &self,
        session_id: String,
        candidate: Option<RTCIceCandidate>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id);
            if session.is_none() {
                Err(SfuError::SessionNotFound.into())
            } else {
                let session = session.unwrap();
                let playground = serde_json::to_string(&SignalingResponse::Candidate(candidate))?;
                session
                    .0
                    .lock()
                    .await
                    .send(Message::from(playground))
                    .await?;
                Ok(())
            }
        })
    }
}

pub fn create_webrtc_state() -> WebrtcState {
    let sessions = Arc::new(Mutex::new(HashMap::new()));
    let signalling = Box::new(WebsocketSignalling::new(Arc::clone(&sessions)));

    let secret_key = {
        let key = env::var_os("SECRET_KEY")
            .expect("Missing SECRET_KEY env variable")
            .to_str()
            .expect("SECRET_KEY contains invalid Unicode")
            .to_string();

        let key = DecodingKey::from_secret(key.as_ref());
        Box::leak(Box::new(key))
    } as SecretKey; // allow SECRET_KEY life endless

    WebrtcState {
        sfu: Sfu::new(signalling),
        sessions: Arc::clone(&sessions),
        secret_key,
    }
}

pub fn create_webrtc_router() -> Router<WebrtcState> {
    Router::new()
        .route("/ws", any(ws))
        .route("/offer", post(accept_offer))
        .route("/answer", post(accept_answer))
        .route("/candidate", post(candidate))
}

async fn ws(
    ws: WebSocketUpgrade,
    Jwt(claims): Jwt,
    State(app_state): State<WebrtcState>,
) -> Result<impl IntoResponse, AppError> {
    let sessions = app_state.sessions.clone();

    let session_id = claims.sub.to_string();

    let resp = ws
        .on_failed_upgrade(move |e| {
            warn!(err:? = e; "Websocket upgrade failed");

            block_on(sessions.lock()).remove(&claims.sub.to_string());
        })
        .on_upgrade(async move |socket| {
            let (sender, receiver) = socket.split();
            let socket_client: Arc<SocketClient> =
                Arc::new((Mutex::new(sender), Mutex::new(receiver)));

            info!(session_id:? = session_id; "Websocket client connected");

            app_state
                .sessions
                .lock()
                .await
                .insert(session_id, Arc::clone(&socket_client));
        });

    Ok(resp)
}

#[derive(Deserialize, Serialize)]
struct AcceptOfferReq {
    offer: RTCSessionDescription,
    room_id: String,
}

#[derive(Deserialize, Serialize)]
struct AnswerResponse {
    answer: RTCSessionDescription,
}

async fn accept_offer(
    Jwt(claims): Jwt,
    State(app_state): State<WebrtcState>,
    Json(req): Json<AcceptOfferReq>,
) -> Result<impl IntoResponse, AppError> {
    let answer = app_state
        .sfu
        .accept_offer(claims.sub.to_string(), req.offer, req.room_id)
        .await?;

    Ok(Json(AnswerResponse { answer }))
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(err:? = self.0; "Failed response");

        if let Some(jwt_err) = self.0.downcast_ref::<jsonwebtoken::errors::Error>() {
            if let ErrorKind::ExpiredSignature = jwt_err.kind() {
                return (StatusCode::UNAUTHORIZED, "token expired".to_string()).into_response();
            }
        };

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Deserialize, Serialize)]
struct AcceptAnswerReq {
    answer: RTCSessionDescription,
    room_id: String,
}

async fn accept_answer(
    Jwt(claims): Jwt,
    State(app_state): State<WebrtcState>,
    Json(req): Json<AcceptAnswerReq>,
) -> Result<impl IntoResponse, AppError> {
    app_state
        .sfu
        .accept_answer(claims.sub.to_string().clone(), req.answer, req.room_id)
        .await?;

    Ok("ok")
}

#[derive(Deserialize, Serialize)]
struct CandidateRequest {
    candidate: RTCIceCandidateInit,
    room_id: String, // TODO remove
}

async fn candidate(
    Jwt(claims): Jwt,
    State(app_state): State<WebrtcState>,
    Json(req): Json<CandidateRequest>,
) -> Result<impl IntoResponse, AppError> {
    app_state
        .sfu
        .accept_candidate(claims.sub.to_string(), req.room_id, req.candidate)
        .await?;

    Ok("ok")
}
