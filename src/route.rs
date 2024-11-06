use std::hash::{DefaultHasher, Hash, Hasher};

use crate::error::RPCResult;
use crate::prelude::*;
use crate::signer::SignerConfig;
use alloy::{
    network::TransactionBuilder,
    primitives::{address, TxKind},
    rlp::Encodable,
    rpc::types::{TransactionInput, TransactionRequest},
};
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
) -> RPCResult<Json<SignReponse>> {
    let rpc_err_map = |e: Error| e.rpc_error(id, jsonrpc.clone());

    {
        let TransactionRequest {
            from, to, input, ..
        } = request.clone();
        let TransactionInput { input, data } = input;
        let input = input.map(|i| i.to_string());
        let data = data.map(|d| d.to_string());

        let from = from.unwrap_or_default();
        let to = to.map(|kind| match kind {
            TxKind::Create => String::from("create"),
            TxKind::Call(addr) => addr.to_string(),
        });
        info!(from = from.to_string(), to, input, data, "sign request");
    };

    if method != "eth_signTransaction" {
        return Err(Error::InvalidRpcMethod(method)).map_err(rpc_err_map);
    }

    let mut req_hash = DefaultHasher::new();
    request.clone().hash(&mut req_hash);

    let gas_price = request.gas_price();

    let tx = request
        .with_gas_price(gas_price.unwrap_or(90000))
        .build_typed_tx()
        .map_err(|_| Error::BuildTransactionError(format!("tx_type is none")))
        .map_err(rpc_err_map)?;

    let mut tx_hash = DefaultHasher::new();
    tx.hash(&mut tx_hash);

    let signature = config.sign_transaction(tx).await.map_err(rpc_err_map)?;

    let mut sign_hash = DefaultHasher::new();
    signature.hash(&mut sign_hash);

    info!(
        request = req_hash.finish(),
        tx = tx_hash.finish(),
        sign = sign_hash.finish(),
        "check hash"
    );

    let mut encoded_sign = Vec::<u8>::new();

    signature.encode(&mut encoded_sign);

    let hex: String = encoded_sign.iter().map(|b| format!("{:02x}", b)).collect();

    Ok(Json(SignReponse {
        id,
        jsonrpc,
        result: format!("0x{}", hex),
    }))
}

pub fn routes(state: SignerConfig) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "OK" }))
        .route("/pub", get(pub_key))
        .route("/", post(sign))
        .with_state(state)
}
