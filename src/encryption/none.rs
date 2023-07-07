use serde::Deserialize;
use futures::stream::BoxStream;
use bytes::Bytes;

use super::encryption::Encryption;

#[derive(Deserialize, Debug)]
pub struct None;

impl Encryption for None {
    fn encrypt(&self, data: BoxStream<Result<Bytes, String>>) -> BoxStream<Result<Bytes, String>> {
        data
    }

    fn decrypt(&self, data: BoxStream<Result<Bytes, String>>) -> BoxStream<Result<Bytes, String>> {
        data
    }
}