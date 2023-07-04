use async_trait::async_trait;
use serde::Deserialize;

use super::aes::Aes;

#[async_trait]
pub trait IEncryption {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, ()>;
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ()>;
}

#[derive(Deserialize, Debug)]
pub enum EncryptionType {
    Aes(Aes),
}

impl EncryptionType {
    pub fn get(&self) -> &dyn IEncryption {
        match self {
            EncryptionType::Aes(encryption) => return encryption
        }
    }
}