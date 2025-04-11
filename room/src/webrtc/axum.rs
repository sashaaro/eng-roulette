use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::extract::ws::{Message, WebSocket};
use axum::routing::{any, post};
use futures::executor::block_on;
use futures::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use crate::webrtc::sfu::{Signalling, SFU};
use futures::{SinkExt, StreamExt};
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use anyhow::{Result};
use axum::middleware::from_extractor;
use http::StatusCode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::errors::ErrorKind;
use log::error;
use thiserror::Error;
use crate::webrtc::extract::{Claims, JWT};

pub(crate) type SocketClient = (Mutex<SplitSink<WebSocket, Message>>, Mutex<SplitStream<WebSocket>>);

#[derive(Clone)]
pub struct AppState {
    pub(crate) sessions: Arc<Mutex<HashMap<String, Arc<SocketClient>>>>,
    pub(crate) sfu: SFU,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "playground")]
pub enum SignalingResponse {
    #[serde(rename = "sdp")]
    Sdp(RTCSessionDescription),

    #[serde(rename = "candidate")]
    Candidate(Option<RTCIceCandidate>),
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

#[derive(Error, Debug)]
pub enum SfuError {
    #[error("Session not found")]
    SessionNotFound,
}

impl Signalling for WebsocketSignalling {
    fn send_sdp(&self, session_id: String, sdp: RTCSessionDescription) -> Pin<Box<dyn Future<Output=Result<()>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id);
            if session.is_none() {
                Err(SfuError::SessionNotFound.into())
            } else {
                let session = session.unwrap();
                let playground = serde_json::to_string(&SignalingResponse::Sdp(sdp))?;
                session.0.lock().await.send(Message::from(playground)).await?;
                Ok(())
            }
        })
    }

    fn send_ice_candidate(&self, session_id: String, candidate: Option<RTCIceCandidate>) -> Pin<Box<dyn Future<Output=Result<()>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id);
            if session.is_none() {
                Err(SfuError::SessionNotFound.into())
            } else {
                let session = session.unwrap();
                let playground = serde_json::to_string(&SignalingResponse::Candidate(candidate))?;
                session.0.lock().await.send(Message::from(playground)).await?;
                Ok(())
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
        .layer(from_extractor::<JWT>())
        .route("/version", any(async || "v0.1.0"))
        .route("/ws", any(ws))
        .route("/offer", post(accept_offer))
        .route("/answer", post(accept_answer))
        .route("/candidate", post(candidate))
        .layer(CorsLayer::permissive())
        .with_state(state);

    app
}

#[derive(Deserialize)]
struct WsRequest {
    jwt: String,
}

async fn ws(
    ws: WebSocketUpgrade,
    Query(req): Query<WsRequest>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {

    let secret = "secret".to_string();

    //let secret = std::env::var("SECRET_KEY").unwrap().as_ref();
    // TODO inject from config
    let decoding_key = &DecodingKey::from_secret(secret.as_ref());


    let jwt = req.jwt.trim_start_matches("Bearer").trim();

    let claims = jsonwebtoken::decode::<Claims>(
        jwt,
        decoding_key,
        &jsonwebtoken::Validation::default(),
    )
        .map(|t| {
            t.claims
        })?;


    let sessions = app_state.sessions.clone();

    let resp =ws.on_failed_upgrade(move |error| {
        println!("on_failed_upgrade {:?}", error);
        block_on(sessions.lock()).remove(&claims.sub.to_string());
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
                .insert(claims.sub.to_string(), Arc::clone(&socket_client));
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
    JWT(claims): JWT,
    State(app_state): State<AppState>,
    Json(req): Json<AcceptOfferReq>,
) -> Result<impl IntoResponse, AppError> {
    let answer = app_state.sfu.accept_offer(claims.sub.to_string(), req.offer, req.room_id).await?;

    Ok(Json(AnswerResponse {
        answer,
    }))
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(err:? = self.0; "Failed response");

        if let Some(jwt_err) = self.0.downcast_ref::<jsonwebtoken::errors::Error>() {
            match jwt_err.kind() {
                ErrorKind::ExpiredSignature => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        format!("token expired"),
                    )
                        .into_response();
                },
                _ => {},
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
    JWT(claims): JWT,
    State(app_state): State<AppState>,
    Json(req): Json<AcceptAnswerReq>
) -> Result<impl IntoResponse, AppError> {
    app_state.sfu.accept_answer(claims.sub.to_string().clone(), req.answer, req.room_id).await?;

    Ok("ok")
}

#[derive(Deserialize, Serialize)]
struct CandidateRequest {
    candidate: RTCIceCandidateInit,
    room_id: String, // TODO remove
}

async fn candidate(
    JWT(claims): JWT,
    State(app_state): State<AppState>,
    Json(req): Json<CandidateRequest>,
) -> Result<impl IntoResponse, AppError> {
    app_state.sfu.accept_candidate(claims.sub.to_string(), req.room_id, req.candidate).await?;

    Ok("ok")
}
