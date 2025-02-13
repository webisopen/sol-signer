mod config;
pub use config::SignerConfig;

use crate::prelude::*;
use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};

use bip39::{Language, Mnemonic};

use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, SeedDerivable, Signer},
};
use solana_signer_gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier};

impl SignerConfig {
    pub async fn signer(&self) -> Result<Box<dyn Signer>> {
        let signer: Box<dyn Signer> = match self {
            Self::PrivateKey(key) => Box::new(Keypair::from_base58_string(&key)),
            SignerConfig::Mnemonic(mnemonic) => {
                let _mnemonic = Mnemonic::from_phrase(&mnemonic, Language::English)
                    .map_err(Error::Bip39Error)?;
                let keypair =
                    Keypair::from_seed(_mnemonic.entropy()).map_err(Error::CustomizeError)?;
                Box::new(keypair)
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

                Box::new(GcpSigner::new(client, key_specifier).await?)
            }
            _ => unimplemented!(),
        };
        Ok(signer)
    }

    // pub async fn wallet(&self) -> Result<EthereumWallet> {
    //     let signer = self.signer().await?;
    //     Ok(EthereumWallet::new(signer))
    // }

    pub async fn address(&self) -> Result<Pubkey> {
        let signer = self.signer().await?;
        signer.try_pubkey().map_err(Error::SignerError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_rlp() {
    //     let rlp = "0xa3f20717a250c2b0b729b7e5becbff67fdaef7e0699da4de7ca5895b02a170a12d887fd3b17bfdce3481f10bea41f45ba9f709d39ce8325427b57afcfc994cee1b";
    //     let tx = TypedTransaction::
    // }
}
