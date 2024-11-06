use crate::prelude::*;
use alloy::{
    consensus::TypedTransaction,
    network::TxSigner,
    primitives::Address,
    signers::{
        aws::AwsSigner,
        gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier},
        local::{coins_bip39::English, LocalSigner, MnemonicBuilder, PrivateKeySigner},
        Signature,
    },
};

use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SignerConfig {
    PrivateKey(String),
    Mnemonic(String),
    KeyStore {
        path: String,
        password: String,
    },
    AzureKeyVault {
        key: String,
        secret: String,
    },
    AwsKms {
        key: String,
    },
    GoogleKms {
        project_id: String,
        location: String,
        key_ring: String,
        key: String,
        version: u64,
    },
    AlicloudKms {
        key: String,
        secret: String,
    },
}

impl SignerConfig {
    pub async fn signer(&self) -> Result<Box<dyn TxSigner<Signature> + Send + Sync + 'static>> {
        let signer: Box<dyn TxSigner<Signature> + Send + Sync + 'static> = match self {
            SignerConfig::PrivateKey(key) => Box::new(key.parse::<PrivateKeySigner>()?),
            SignerConfig::Mnemonic(mnemonic) => Box::new(
                MnemonicBuilder::<English>::default()
                    .phrase(mnemonic)
                    .build()?,
            ),
            SignerConfig::KeyStore { path, password } => {
                Box::new(LocalSigner::decrypt_keystore(path, password)?)
            }
            SignerConfig::AwsKms { key } => {
                let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
                let client = aws_sdk_kms::Client::new(&config);
                Box::new(AwsSigner::new(client, key.clone(), Some(1)).await?)
            }
            Self::GoogleKms {
                project_id,
                location,
                key_ring,
                key,
                version,
            } => {
                let keyring_ref = GcpKeyRingRef::new(&project_id, &location, &key_ring);

                let client = GoogleApi::from_function(
                    KeyManagementServiceClient::new,
                    "https://cloudkms.googleapis.com",
                    None,
                )
                .await?;
                let key_specifier = KeySpecifier::new(keyring_ref, key, *version);

                Box::new(GcpSigner::new(client, key_specifier, None).await?)
            }
            _ => unimplemented!(),
        };
        Ok(signer)
    }

    pub async fn sign_transaction(&self, tx: TypedTransaction) -> Result<Signature> {
        let signer = self.signer().await?;

        match tx {
            TypedTransaction::Eip1559(tx) => {
                let mut tx_1559 = tx.clone();

                signer
                    .sign_transaction(&mut tx_1559)
                    .await
                    .map_err(Error::SignerError)
            }
            TypedTransaction::Eip4844(tx) => {
                let mut tx_4844 = tx.clone();

                signer
                    .sign_transaction(&mut tx_4844)
                    .await
                    .map_err(Error::SignerError)
            }
            tx => Err(Error::InvalidTransactionType(tx.tx_type().to_string())),
        }
    }

    pub async fn address(&self) -> Result<Address> {
        let signer = self.signer().await?;
        Ok(signer.address())
    }
}
