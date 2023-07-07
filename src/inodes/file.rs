use std::sync::Arc;

use async_trait::async_trait;
use futures::{StreamExt, stream::BoxStream};
use serde::{Serialize, Deserialize};

use crate::{blocks::{indirect_block::IndirectBlock, block::{Block, BlockType}}, global::Global};
use super::{inode::Inode, metadata::Metadata};


#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub data: IndirectBlock,
    pub metadata: Metadata
}

#[async_trait]
impl Inode for File {
    async fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    async fn delete(&mut self, global: Arc<Global>) {
        self.data.delete(global).await;
    }
}

impl File {
    async fn create(global: Arc<Global>, data: Vec<u8>) -> Result<Self, String> {
        let block = match IndirectBlock::create(global, data, 0).await? {
            BlockType::Indirect(block) => block,
            _ => panic!("This should never happen"),
        };
        let metadata = Metadata::new();
        Ok(Self {
            data: block,
            metadata
        })
    }

    fn get(&self, global: Arc<Global>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            let range = self.data.range(global.clone()).await?;
            let mut stream = self.data.get(global.clone(), range.clone());
            while let Some(result) = stream.next().await {
                yield result;
            }
        })
    }
}