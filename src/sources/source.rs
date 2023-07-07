use async_trait::async_trait;
use serde::Deserialize;

use super::local::LocalSource;

#[async_trait]
pub trait Source {
    async fn get(&self, descriptor: String) -> Result<Vec<u8>, String>;
    async fn put(&self, descriptor: String, data: Vec<u8>) -> Result<(), String>;
    async fn delete(&self, descriptor: String) -> Result<(), String>;
    async fn create(&self) -> Result<String, String>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SourceType {
    #[serde(rename = "local")]
    LocalSource(LocalSource)   
}

// This macro removes the need to write out the match statement for each method in the enum
macro_rules! match_method {
    ($self:ident, $method:ident, $($arg:expr),*) => {
        match $self {
            SourceType::LocalSource(source) => source.$method($($arg),*)
        }
    };
}

#[async_trait]
impl Source for SourceType {
    async fn get(&self, descriptor: String) -> Result<Vec<u8>, String> {
        match_method!(self, get, descriptor).await
    }

    async fn put(&self, descriptor: String, data: Vec<u8>) -> Result<(), String> {
        match_method!(self, put, descriptor, data).await
    }

    async fn delete(&self, descriptor: String) -> Result<(), String> {
        match_method!(self, delete, descriptor).await
    }

    async fn create(&self) -> Result<String, String> {
        match_method!(self, create, ).await
    }
}