use alloy::{
    network::Ethereum,
    signers::{aws::AwsSignerError, gcp::GcpSignerError, local::LocalSignerError},
};
use axum::response::{IntoResponse, Response};
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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}
