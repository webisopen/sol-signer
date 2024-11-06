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
