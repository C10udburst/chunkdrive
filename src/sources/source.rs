use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use serde::Deserialize;

use super::local::LocalSource;

#[async_trait]
pub trait Source {
    // Takes a descriptor and returns a stream of data or an error (String)
    fn get(&self, descriptor: String) -> BoxStream<Result<Bytes, String>>;
    // Takes a descriptor and data and uploads the data to the descriptor or returns an error (String)
    async fn put(&self, descriptor: String, data: Bytes) -> Result<(), String>;
    // Takes a descriptor and deletes the data at the descriptor or returns an error (String)
    async fn delete(&self, descriptor: String) -> Result<(), String>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SourceType {
    #[serde(rename = "local")]
    LocalSource(LocalSource)   
}