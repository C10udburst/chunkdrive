use serde::Deserialize;
use super::source::Source;
use super::error::SourceError;
use async_trait::async_trait;

#[derive(Deserialize, Debug)]
pub struct DiscordWebhook {
    pub url: String,
}

#[async_trait]
impl Source for DiscordWebhook {
    async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError> {
        unimplemented!()
    }

    async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError> {
        unimplemented!()
    }

    async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError> {
        unimplemented!()
    }

    async fn create(&self) -> Result<Vec<u8>, SourceError> {
        unimplemented!()
    }
}