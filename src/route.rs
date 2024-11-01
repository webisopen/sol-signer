use crate::prelude::*;
use crate::signer::SignerConfig;
use alloy::{
    eips::eip2718::Encodable2718, network::TransactionBuilder, primitives::U256,
    rpc::types::TransactionRequest,
};

use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

async fn get_config(State(config): State<SignerConfig>) -> impl IntoResponse {
    axum::Json(config)
}

async fn pub_key(config: State<SignerConfig>) -> impl IntoResponse {
    let addr = config.address().await.unwrap();
    format!("{:?}", addr)
}

#[derive(Debug, Deserialize)]
struct SignRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: [TransactionRequest; 1],
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
) -> Result<Json<SignReponse>> {
    if method != "eth_sendTransaction" {
        return Err(Error::InvalidRpcMethod(method));
    }

    let wallet = config.wallet().await?;
    let tx_envelop = request
        .with_value(U256::from(100))
        .with_nonce(20)
        .with_gas_price(600_000_000)
        .with_gas_limit(1_000_000_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000)
        .with_chain_id(57770793173)
        .build(&wallet)
        .await?;

    let mut encoded_tx = Vec::<u8>::new();

    tx_envelop.encode_2718(&mut encoded_tx);

    let hex_string: String = encoded_tx.iter().map(|b| format!("{:02x?}", b)).collect();

    Ok(Json(SignReponse {
        id,
        jsonrpc,
        result: format!("0x{}", hex_string),
    }))
}

pub fn routes(state: SignerConfig) -> Router {
    Router::new()
        .route("/config", get(get_config))
        .route("/pub", get(pub_key))
        .route("/", post(sign))
        .with_state(state)
}
