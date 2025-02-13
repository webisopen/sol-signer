use std::hash::{DefaultHasher, Hash, Hasher};

use crate::error::RPCResult;
use crate::prelude::*;
use crate::signer::SignerConfig;
use solana_sdk::transaction::Transaction;
use tracing::info;

use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

async fn pub_key(config: State<SignerConfig>) -> Result<String> {
    let addr = config.address().await?;
    Ok(addr.to_string())
}

#[derive(Debug, Deserialize)]
struct SignRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: [Transaction; 1],
}

#[derive(Debug, Serialize)]
struct SignReponse {
    id: u64,
    jsonrpc: String,
    result: String,
}

async fn sign(
    config: State<SignerConfig>,
    Json(SignRequest {
        id,
        jsonrpc,
        method,
        params: [request],
    }): Json<SignRequest>,
) -> RPCResult<Json<SignReponse>> {
    let rpc_err_map = |e: Error| e.rpc_error(id, jsonrpc.clone());

    if method != "signTransaction" {
        return Err(Error::InvalidRpcMethod(method)).map_err(rpc_err_map);
    }

    let req_hash = request.message().hash();

    let signer = config.signer().await.map_err(rpc_err_map)?;

    let signature = signer.sign_message(&request.message_data());

    let mut tx_hash = DefaultHasher::new();
    signature.hash(&mut tx_hash);

    info!(
        req_hash = req_hash.to_string(),
        tx_hash = tx_hash.finish(),
        "sign tx"
    );

    Ok(Json(SignReponse {
        id,
        jsonrpc,
        result: signature.to_string(),
    }))
}

pub fn routes(state: SignerConfig) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "OK" }))
        .route("/pub", get(pub_key))
        .route("/", post(sign))
        .with_state(state)
}
