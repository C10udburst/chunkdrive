use async_trait::async_trait;
use serde::Deserialize;

use crate::global::Descriptor;

use super::{local::LocalSource, discord_webhook::DiscordWebhook, github_releases::GithubReleases};

#[async_trait]
pub trait Source {
    fn max_size(&self) -> usize;
    async fn get(&self, descriptor: &Descriptor) -> Result<Vec<u8>, String>;
    async fn put(&self, descriptor: &Descriptor, data: Vec<u8>) -> Result<(), String>;
    async fn delete(&self, descriptor: &Descriptor) -> Result<(), String>;
    async fn create(&self) -> Result<Descriptor, String>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SourceType {
    #[serde(rename = "local")]
    LocalSource(LocalSource),
    #[serde(rename = "discord_webhook")]
    DiscordWebhook(DiscordWebhook),
    #[serde(rename = "github_releases")]
    GithubRelease(GithubReleases)
}

// This macro removes the need to write out the match statement for each method in the enum
macro_rules! match_method {
    ($self:ident, $method:ident, $($arg:expr),*) => {
        match $self {
            SourceType::LocalSource(source) => source.$method($($arg),*),
            SourceType::DiscordWebhook(source) => source.$method($($arg),*),
            SourceType::GithubRelease(source) => source.$method($($arg),*),
        }
    };
}

impl SourceType {
    pub fn human_readable(&self) -> &str {
        match self {
            SourceType::LocalSource(_) => "local folder",
            SourceType::DiscordWebhook(_) => "discord webhook",
            SourceType::GithubRelease(_) => "github releases",
        }
    }
}

#[async_trait]
impl Source for SourceType {
    fn max_size(&self) -> usize {
        match_method!(self, max_size, )
    }

    async fn get(&self, descriptor: &Descriptor) -> Result<Vec<u8>, String> {
        match_method!(self, get, descriptor).await
    }

    async fn put(&self, descriptor: &Descriptor, data: Vec<u8>) -> Result<(), String> {
        match_method!(self, put, descriptor, data).await
    }

    async fn delete(&self, descriptor: &Descriptor) -> Result<(), String> {
        match_method!(self, delete, descriptor).await
    }

    async fn create(&self) -> Result<Descriptor, String> {
        match_method!(self, create, ).await
    }
}