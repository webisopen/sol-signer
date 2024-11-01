mod config;
mod error;
mod prelude;
mod route;
mod signer;

use axum::Router;
use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> prelude::Result<()> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let lisenter = TcpListener::bind(&addr).await.unwrap();

    let args = config::SignerOpts::parse();

    let signer_config: signer::SignerConfig = args.try_into()?;

    let routes = route::routes(signer_config.clone());
    let app = Router::new().merge(routes);

    axum::serve(
        lisenter,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}
