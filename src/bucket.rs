/*
    Bucket is an abstraction over a source, it includes additional features like encryption or caching
*/

use serde::Deserialize;

use crate::{sources::source::{SourceType, Source}, encryption::encryption::{EncryptionType, Encryption}};

#[derive(Deserialize, Debug)]
pub struct Bucket {
    source: SourceType,
    #[serde(default)]
    encryption: EncryptionType,
}

impl Bucket {
    // Takes a descriptor and returns a stream of data or an error (String)
    async fn get(&self, descriptor: String) -> Result<Vec<u8>, String> {
        let iv = descriptor.as_bytes().to_vec();
        let data = self.source.get(descriptor).await?;
        let decrypted = self.encryption.decrypt(data, iv)?;
        Ok(decrypted)
    }
    
    // Takes a descriptor and data and uploads the data to the descriptor or returns an error (String)
    async fn put(&self, descriptor: String, data: Vec<u8>) -> Result<(), String> {
        let iv = descriptor.as_bytes().to_vec();
        let encrypted = self.encryption.encrypt(data, iv)?;
        self.source.put(descriptor, encrypted).await
    }
    
    // Takes a descriptor and deletes the data at the descriptor or returns an error (String)
    async fn delete(&self, descriptor: String) -> Result<(), String> {
        self.source.delete(descriptor).await
    }

    // Creates a new descriptor and returns it or returns an error (String)
    async fn create(&self) -> Result<String, String> {
        self.source.create().await
    }
}