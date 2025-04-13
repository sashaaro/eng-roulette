use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

mod webrtc;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value_t = 8082)]
    pub port: u16,

    #[arg(short, long, default_value_t = false)]
    pub webrtc: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new()
        .filter(None, LevelFilter::Info)
        .filter(Some("webrtc::peer_connection"), LevelFilter::Warn)
        .filter(Some("webrtc_ice::mdns"), LevelFilter::Warn)
        .filter(Some("webrtc_mdns::conn"), LevelFilter::Warn)
        .filter(
            Some("webrtc_ice::agent::agent_internal"),
            LevelFilter::Warn,
        )
        .filter(Some("webrtc_ice::agent::agent_gather"), LevelFilter::Warn)
        .filter(Some("webrtc_srtp::session"), LevelFilter::Warn)
        .init();

    let args = Args::parse();
    let app = webrtc::axum::create_webrtc_router().await;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    Ok(axum::serve(listener, app).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webrtc::axum::create_webrtc_router;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn hello_world() {
        let app = create_webrtc_router().await;

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}