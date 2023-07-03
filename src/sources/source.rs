use async_trait::async_trait;
use serde::Deserialize;
use super::error::SourceError;

use super::discord_webhook::DiscordWebhook;
use super::local::Local;

#[async_trait]
pub trait Source {
    async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError>;
    async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError>;
    async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError>;
    async fn create(&self) -> Result<Vec<u8>, SourceError>;
}

#[derive(Deserialize, Debug)]
pub enum SourceType {
    DiscordWebhook(DiscordWebhook),
    Local(Local),
}

impl SourceType {
    pub fn get(&self) -> Option<&dyn Source> {
        match self {
            SourceType::DiscordWebhook(source) => Some(source),
            SourceType::Local(source) => Some(source)
        }
    }
}