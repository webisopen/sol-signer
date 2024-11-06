mod config;
mod error;
mod prelude;
mod route;
mod signer;

use axum::Router;
use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;

use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> prelude::Result<()> {
    let args = config::SignerOpts::parse();

    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()),
    );

    if args.debug {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
            .init();
    }

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let lisenter = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    let signer_config: signer::SignerConfig = args.try_into()?;

    let routes = route::routes(signer_config.clone());
    let app = Router::new().merge(routes);

    axum::serve(
        lisenter,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
