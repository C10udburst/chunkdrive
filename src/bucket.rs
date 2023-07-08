/*
    Bucket is an abstraction over a source, it includes additional features like encryption or caching.
    Each bucket has a maximum size, which is the maximum size of a single data chunk that can be stored in the bucket.
*/

use serde::Deserialize;

use crate::{sources::source::{SourceType, Source}, encryption::encryption::{EncryptionType, Encryption}, global::Descriptor};

#[derive(Deserialize, Debug)]
pub struct Bucket {
    source: SourceType,
    #[serde(default)]
    encryption: EncryptionType,
}

impl Bucket {
    // Returns the maximum size of data that can be stored in a single descriptor
    pub fn max_size(&self) -> usize {
        self.encryption.max_size(
            self.source.max_size()
        )
    }

    // Takes a descriptor and returns a stream of data or an error (String)
    pub async fn get(&self, descriptor: &Descriptor) -> Result<Vec<u8>, String> {
        let iv = descriptor.to_vec();
        let data = self.source.get(descriptor).await?;
        let decrypted = self.encryption.decrypt(data, iv)?;
        Ok(decrypted)
    }
    
    // Takes a descriptor and data and uploads the data to the descriptor or returns an error (String)
    pub async fn put(&self, descriptor: &Descriptor, data: Vec<u8>) -> Result<(), String> {
        let iv = descriptor.to_vec();
        let encrypted = self.encryption.encrypt(data, iv)?;
        self.source.put(descriptor, encrypted).await
    }
    
    // Takes a descriptor and deletes the data at the descriptor or returns an error (String)
    pub async fn delete(&self, descriptor: &Descriptor) -> Result<(), String> {
        self.source.delete(descriptor).await
    }

    // Creates a new descriptor and returns it or returns an error (String)
    pub async fn create(&self) -> Result<Descriptor, String> {
        self.source.create().await
    }
}