use serde::Deserialize;

use crate::sources::source::SourceType;
use crate::sources::error::SourceError;
use crate::encryption::encryption::EncryptionType;

#[derive(Deserialize, Debug)]
pub struct Source {
    pub source: SourceType,
    pub encryption: Option<EncryptionType>,
}

impl Source {
    pub async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError> {
        let encrypted = match self.source.get().get(descriptor).await {
            Ok(encrypted) => encrypted,
            Err(error) => return Err(error),
        };
        match &self.encryption {
            Some(encryption) => {
                let decrypted = encryption.get().decrypt(&encrypted).await.unwrap();
                return Ok(decrypted);
            },
            None => return Ok(encrypted),
        }
    }

    pub async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError> {
        let encrypted = match &self.encryption {
            Some(encryption) => encryption.get().encrypt(data).await.unwrap(),
            None => data.to_vec(),
        };
        match self.source.get().put(descriptor, &encrypted).await {
            Ok(_) => return Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError> {
        match self.source.get().delete(descriptor).await {
            Ok(_) => return Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn create(&self, data: &[u8]) -> Result<Vec<u8>, SourceError> {
        let encrypted = match &self.encryption {
            Some(encryption) => encryption.get().encrypt(data).await.unwrap(),
            None => data.to_vec(),
        };
        match self.source.get().create(&encrypted).await {
            Ok(descriptor) => return Ok(descriptor),
            Err(error) => return Err(error),
        }
    }
}