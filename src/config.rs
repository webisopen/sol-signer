use crate::prelude::*;
use clap::Parser;

use crate::signer::SignerConfig;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct SignerOpts {
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(name = "type", short = 't', long, env = "SIGNER_TYPE")]
    _type: String,

    #[arg(long, env = "SIGNER_PRIVATE_KEY")]
    private_key: Option<String>,

    #[arg(long, env = "SIGNER_MNEMONIC")]
    mnemonic: Option<String>,

    #[arg(long = "keystore.path", env = "SIGNER_KEYSTORE_PATH")]
    keystore_path: Option<String>,
    #[arg(long = "keystore.password", env = "SIGNER_KEYSTORE_PASSWORD")]
    keystore_password: Option<String>,

    #[arg(long = "azurekeyvault.key", env = "SIGNER_AZUREKEYVAULT_KEY")]
    azurekeyvault_key: Option<String>,

    #[arg(long = "awskms.key", env = "SIGNER_AWSKMS_KEY")]
    awskms_key: Option<String>,

    #[arg(long = "gcpkms.project_id", env = "SIGNER_GCPKMS_PROJECT_ID")]
    gcpkms_project_id: Option<String>,
    #[arg(long = "gcpkms.location", env = "SIGNER_GCPKMS_LOCATION")]
    gcpkms_location: Option<String>,
    #[arg(long = "gcpkms.key_ring", env = "SIGNER_GCPKMS_KEY_RING")]
    gcpkms_key_ring: Option<String>,
    #[arg(long = "gcpkms.key", env = "SIGNER_GCPKMS_KEY")]
    gcpkms_key: Option<String>,
    #[arg(long = "gcpkms.version", env = "SIGNER_GCPKMS_VERSION")]
    gcpkms_version: Option<u64>,
}

impl TryInto<SignerConfig> for SignerOpts {
    type Error = Error;

    fn try_into(self) -> Result<SignerConfig> {
        match self._type.as_str() {
            "private_key" => Ok(SignerConfig::PrivateKey(
                self.private_key
                    .ok_or(Error::RequireConfigKeyNotFound("private_key"))?,
            )),
            "mnemonic" => Ok(SignerConfig::Mnemonic(
                self.mnemonic
                    .ok_or(Error::RequireConfigKeyNotFound("mnemonic"))?,
            )),
            "keystore" => Ok(SignerConfig::KeyStore {
                path: self
                    .keystore_path
                    .ok_or(Error::RequireConfigKeyNotFound("keystore.path"))?,
                password: self
                    .keystore_password
                    .ok_or(Error::RequireConfigKeyNotFound("keytore.password"))?,
            }),
            "gcpkms" => Ok(SignerConfig::GoogleKms {
                project_id: self
                    .gcpkms_project_id
                    .ok_or(Error::RequireConfigKeyNotFound("gcpkms.project_id"))?,
                location: self
                    .gcpkms_location
                    .ok_or(Error::RequireConfigKeyNotFound("gcpkms.location"))?,
                key_ring: self
                    .gcpkms_key_ring
                    .ok_or(Error::RequireConfigKeyNotFound("gcpkms.key_ring"))?,
                key: self
                    .gcpkms_key
                    .ok_or(Error::RequireConfigKeyNotFound("gcpkms.key"))?,
                version: self
                    .gcpkms_version
                    .ok_or(Error::RequireConfigKeyNotFound("gcpkms.version"))?,
            }),
            _ => Err(Error::InvalidSignerType(self._type)),
        }
    }
}
