/*
    This module implements Stored object, which serializes and deserializes objects to and from the database.
    It has no knowledge of the data types, so make sure to use the correct type when deserializing.
    It uses messagepack for serialization for backwards compatibility.
 */

use std::sync::Arc;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use rmp_serde::{Serializer, Deserializer};
use crate::{global::Global, blocks::{indirect_block::IndirectBlock, block::{Block, BlockType}}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stored {
    #[serde(rename = "d")]
    pub data: IndirectBlock
}

impl Stored {
    pub async fn get<T: Deserialize<'static>>(&self, global: Arc<Global>) -> Result<T, String> {
        let range = self.data.range(global.clone()).await?;
        let mut stream = self.data.get(global, range);
        let mut data = Vec::new();
        while let Some(chunk) = stream.next().await {
            data.extend(chunk?);
        }
        let mut deserializer = Deserializer::new(&data[..]);
        Ok(Deserialize::deserialize(&mut deserializer).unwrap())
    }

    pub async fn put<T: Serialize>(&mut self, global: Arc<Global>, data: T) -> Result<(), String> {
        let mut serializer = Serializer::new(Vec::new());
        data.serialize(&mut serializer).unwrap();
        let data = serializer.into_inner();
        let range = 0..data.len();
        self.data.put(global, data, range).await?;
        Ok(())
    }

    pub async fn create<T: Serialize>(global: Arc<Global>, data: T) -> Result<Stored, String> {
        let mut serializer = Serializer::new(Vec::new());
        data.serialize(&mut serializer).unwrap();
        let data = serializer.into_inner();
        let block = match IndirectBlock::create(global.clone(), data, 0).await? {
            BlockType::Indirect(block) => block,
            _ => panic!("This should never happen")
        };
        Ok(Stored {
            data: block
        })
    }

    pub async fn delete(&self, global: Arc<Global>) {
        self.data.delete(global).await;
    }
}