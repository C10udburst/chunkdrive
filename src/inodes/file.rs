use std::sync::Arc;

use async_trait::async_trait;
use futures::{StreamExt, stream::BoxStream};
use serde::{Serialize, Deserialize};

use crate::{blocks::{indirect_block::IndirectBlock, block::{Block, BlockType}}, global::Global};
use super::{inode::{Inode, InodeType}, metadata::{Metadata, Size}};


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
        self.data.delete(global).await.ok(); // TODO: handle errors
    }
}

impl File {
    pub fn to_enum(self) -> InodeType {
        InodeType::File(self)
    }

    pub async fn create(global: Arc<Global>, data: Vec<u8>) -> Result<Self, String> {
        let size = data.len();
        let block = match IndirectBlock::create(global, data, 0).await? {
            BlockType::Indirect(block) => block,
            _ => panic!("This should never happen"),
        };
        let mut metadata = Metadata::new();
        metadata.size = Size::Bytes(size);
        Ok(Self {
            data: block,
            metadata
        })
    }

    pub fn get(&self, global: Arc<Global>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            let range = self.data.range(global.clone()).await?;
            let mut stream = self.data.get(global.clone(), range.clone());
            while let Some(result) = stream.next().await {
                yield result;
            }
        })
    }
}