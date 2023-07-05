use async_trait::async_trait;
use serde::Deserialize;
use super::error::SourceError;

use super::discord_webhook::DiscordWebhook;
use super::local::Local;

#[async_trait]
pub trait ISource {
    fn max_size(&self) -> usize;
    async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError>;
    async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError>;
    async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError>;
    async fn create(&self, data: &[u8]) -> Result<Vec<u8>, SourceError>;
}

#[derive(Deserialize, Debug)]
pub enum SourceType {
    DiscordWebhook(DiscordWebhook),
    Local(Local),
}

impl SourceType {
    pub fn as_dyn(&self) -> &dyn ISource {
        match self {
            SourceType::DiscordWebhook(source) => source,
            SourceType::Local(source) => source,
        }
    }    
}

macro_rules! impl_method {
    ($method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl SourceType {
            pub async fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    SourceType::DiscordWebhook(source) => source.$method($($arg),*).await,
                    SourceType::Local(source) => source.$method($($arg),*).await,
                }
            }
        }
    };
    (sync $method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl SourceType {
            pub fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    SourceType::DiscordWebhook(source) => source.$method($($arg),*),
                    SourceType::Local(source) => source.$method($($arg),*),
                }
            }
        }
    };
}

impl_method!(sync max_size, () -> usize);
impl_method!(get, (descriptor: &[u8]) -> Result<Vec<u8>, SourceError>);
impl_method!(put, (descriptor: &[u8], data: &[u8]) -> Result<(), SourceError>);
impl_method!(delete, (descriptor: &[u8]) -> Result<(), SourceError>);
impl_method!(create, (data: &[u8]) -> Result<Vec<u8>, SourceError>);