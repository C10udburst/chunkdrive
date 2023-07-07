use serde::Deserialize;
use futures::stream::BoxStream;
use bytes::Bytes;

pub trait Encryption {
    // Takes a stream of data and returns a stream of encrypted data or an error (String)
    fn encrypt(&self, data: BoxStream<Result<Bytes, String>>) -> BoxStream<Result<Bytes, String>>;
    // Takes a stream of encrypted data and returns a stream of decrypted data or an error (String)
    fn decrypt(&self, data: BoxStream<Result<Bytes, String>>) -> BoxStream<Result<Bytes, String>>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum EncryptionType {
    #[serde(rename = "none")]
    None,
}

impl Default for EncryptionType {
    fn default() -> Self {
        EncryptionType::None
    }
}