use clap::Parser;

mod webrtc;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value_t = 8082
    )]
    pub port: u16,

    #[arg(short, long, default_value_t = false)]
    pub webrtc: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Builder::new()
    //     .filter(None, LevelFilter::Off) // Disable all logs
    //     .init();

    let args = Args::parse();
    let app = webrtc::axum::create_sfu_router().await;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    Ok(axum::serve(listener, app).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::{Service, ServiceExt};
    use crate::webrtc::axum::create_sfu_router;

    #[tokio::test]
    async fn hello_world() {
        let app = create_sfu_router().await;

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/version").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}