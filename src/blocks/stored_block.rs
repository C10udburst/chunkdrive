/*
    This block type uses Stored to store the description of a block it wraps.
 */

use std::{sync::Arc, ops::Range};
use async_trait::async_trait;
use futures::{StreamExt, stream::BoxStream};
use serde::{Serialize, Deserialize};

use crate::{global::Global, blocks::block::{Block, BlockType}, stored::Stored};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredBlock {
    #[serde(rename = "s")]
    pub stored: Stored
}

#[async_trait]
impl Block for StoredBlock {
    async fn range(&self, global: Arc<Global>) -> Result<Range<usize>, String> {
        self.stored.get::<BlockType>(global.clone()).await?.range(global).await
    }

    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, range: Range<usize>) -> Result<(), String> {
        let mut block = self.stored.get::<BlockType>(global.clone()).await?;
        block.put(global.clone(), data, range).await?;
        self.stored.put(global, block).await
    }

    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            let global = global.clone();
            let block = self.stored.get::<BlockType>(global.clone()).await?;
            let mut stream = block.get(global, range.clone());
            while let Some(chunk) = stream.next().await {
                yield chunk;
            }
        })
    }

    async fn delete(&self, global: Arc<Global>) {
        self.stored.get::<BlockType>(global.clone()).await.unwrap().delete(global.clone()).await;
        self.stored.delete(global).await
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        let block = BlockType::create(global.clone(), data, start).await?;
        let stored = Stored::create(global.clone(), block).await?;
        Ok(BlockType::Stored(StoredBlock {
            stored
        }))
    }

    async fn repair(&self, global: Arc<Global>, range: Range<usize>) -> Result<(), String> {
        self.stored.get::<BlockType>(global.clone()).await?.repair(global, range).await
    }

    fn into(self) -> BlockType {
        BlockType::Stored(self)
    }
}