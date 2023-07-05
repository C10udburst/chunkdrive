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
    pub fn as_dyn(&self) -> &dyn IEncryption {
        match self {
            EncryptionType::Aes(encryption) => encryption,
        }
    }
}

macro_rules! impl_method {
    ($method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl EncryptionType {
            pub async fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    EncryptionType::Aes(encryption) => encryption.$method($($arg),*).await,
                }
            }
        }
    };
}

impl_method!(encrypt, (data: &[u8]) -> Result<Vec<u8>, ()>);
impl_method!(decrypt, (data: &[u8]) -> Result<Vec<u8>, ()>);