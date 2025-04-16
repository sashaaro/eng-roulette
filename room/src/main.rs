use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use tower_http::cors::CorsLayer;

mod extract;
mod webrtc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 8082)]
    pub port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new()
        .filter(None, LevelFilter::Info)
        .filter(Some("webrtc::peer_connection"), LevelFilter::Warn)
        .filter(Some("webrtc_ice::mdns"), LevelFilter::Warn)
        .filter(Some("webrtc_mdns::conn"), LevelFilter::Warn)
        .filter(Some("webrtc_ice::agent::agent_internal"), LevelFilter::Warn)
        .filter(Some("webrtc_ice::agent::agent_gather"), LevelFilter::Warn)
        .filter(Some("webrtc_srtp::session"), LevelFilter::Warn)
        .init();

    let args = Args::parse();

    let webrtc_state = webrtc::axum::create_webrtc_state();

    let app = webrtc::axum::create_webrtc_router()
        .with_state(webrtc_state)
        .layer(CorsLayer::permissive()) // TODO
    ;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    Ok(axum::serve(listener, app).await?)
}
