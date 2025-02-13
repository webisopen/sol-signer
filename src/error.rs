use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use solana_signer_gcp::GcpSignerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Customize error: {0}")]
    CustomizeError(Box<dyn std::error::Error>),

    #[error(transparent)]
    SignerError(#[from] solana_sdk::signer::SignerError),

    #[error(transparent)]
    Bip39Error(#[from] bip39::ErrorKind),

    #[error(transparent)]
    GcpSignerError(#[from] GcpSignerError),

    #[error(transparent)]
    GcloudSDKError(#[from] gcloud_sdk::error::Error),

    #[error("Invalid signer type '{0}'")]
    InvalidSignerType(String),

    #[error("Require config key '{0}' not found")]
    RequireConfigKeyNotFound(&'static str),

    #[error("Invalid rpc method '{0}'")]
    InvalidRpcMethod(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.to_string(),
        )
            .into_response()
    }
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

        let err_string = self.error.to_string();
        tracing::info!(err_string);

        {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrResponse {
                    id: self.id,
                    jsonrpc: self.jsonrpc,
                    error: err_string,
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
