use alloy::{
    network::Ethereum,
    signers::{aws::AwsSignerError, gcp::GcpSignerError, local::LocalSignerError},
};
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SignerError(#[from] alloy::signers::Error),

    #[error(transparent)]
    LocalSignerError(#[from] LocalSignerError),

    #[error(transparent)]
    AwsSignerError(#[from] AwsSignerError),

    #[error(transparent)]
    GcpSignerError(#[from] GcpSignerError),

    #[error(transparent)]
    GcloudSDKError(#[from] gcloud_sdk::error::Error),

    #[error(transparent)]
    TransactionBuilderError(#[from] alloy::network::TransactionBuilderError<Ethereum>),

    #[error("Invalid signer type '{0}'")]
    InvalidSignerType(String),

    #[error("Require config key '{0}' not found")]
    RequireConfigKeyNotFound(&'static str),

    #[error("Invalid rpc method '{0}'")]
    InvalidRpcMethod(String),
}

#[derive(Debug)]
pub struct RPCError {
    id: u64,
    jsonrpc: String,
    error: Error,
}

pub type RPCResult<T> = std::result::Result<T, RPCError>;

impl IntoResponse for RPCError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrResponse {
            id: u64,
            jsonrpc: String,
            error: String,
        }

        {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrResponse {
                    id: self.id,
                    jsonrpc: self.jsonrpc,
                    error: self.error.to_string(),
                }),
            )
                .into_response()
        }
    }
}

impl Error {
    pub fn rpc_error(self, id: u64, jsonrpc: String) -> RPCError {
        RPCError {
            id,
            jsonrpc,
            error: self,
        }
    }
}
