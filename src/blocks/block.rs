/*
    This module contains the logic for the block.
    Blocks split the data into chunks and store each chunk in a random bucket or buckets (depending on the redundancy).
    The block should know how to reassemble the chunks into the original data.
    The block should be also able to detect and repair missing chunks if redundancy is enabled.
    The block should also know which range of bytes it contains.
 */

use std::{ops::Range, sync::Arc};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Serialize, Deserialize};

use crate::global::Global;
use super::{direct_block::DirectBlock, indirect_block::IndirectBlock, stored_block::StoredBlock};

#[async_trait]
pub trait Block {
    async fn range(&self, global: Arc<Global>) -> Result<Range<usize>, String>;
    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>>;
    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, range: Range<usize>) -> Result<(), String>;
    async fn delete(&self, global: Arc<Global>);
    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String>;
    async fn repair(&self, global: Arc<Global>, range: Range<usize>) -> Result<(), String>;
    fn into(self) -> BlockType;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockType {
    #[serde(rename = "d")]
    Direct(DirectBlock),
    #[serde(rename = "i")]
    Indirect(IndirectBlock),
    #[serde(rename = "s")]
    Stored(StoredBlock),
} // we use short names to reduce the size of the serialized data while allowing backwards compatibility

macro_rules! match_method {
    ($self:ident, $method:ident, $($arg:expr),*) => {
        match $self {
            BlockType::Direct(block) => block.$method($($arg),*),
            BlockType::Indirect(block) => block.$method($($arg),*),
            BlockType::Stored(block) => block.$method($($arg),*),
        }
    };
}

#[async_trait]
impl Block for BlockType {
    async fn range(&self, global: Arc<Global>) -> Result<Range<usize>, String> {
        match_method!(self, range, global).await
    }

    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>> {
        match_method!(self, get, global, range)
    }

    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, range: Range<usize>) -> Result<(), String> {
        match_method!(self, put, global, data, range).await
    }

    async fn delete(&self, global: Arc<Global>) {
        match_method!(self, delete, global).await
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        IndirectBlock::create(global, data, start).await // we use indirect blocks, because they will fit any data size
    }

    async fn repair(&self, global: Arc<Global>, range: Range<usize>) -> Result<(), String> {
        match_method!(self, repair, global, range).await
    }

    fn into(self) -> BlockType {
        self
    }
}

#[cfg(test)]
mod block_tests {
    use std::sync::Arc;
    use futures::StreamExt;
    use serde_yaml::from_str;

    use crate::{blocks::block::{Block, BlockType}, global::Global, tests::make_temp_config};

    #[tokio::test]
    async fn test_block() {
        let global = Arc::new(from_str::<Global>(&make_temp_config(false, 1024*25)).unwrap());
        dbg!(&global);
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7].repeat(1000);
        let range = 0..data.len();
        let mut block = BlockType::create(global.clone(), data.clone(), 0).await.unwrap();
        assert_eq!(block.range(global.clone()).await.unwrap(), range);
        let mut got1 = Vec::new();
        {
            let mut stream = block.get(global.clone(), range.clone());
            while let Some(chunk) = stream.next().await {
                got1.extend(chunk.unwrap());
            }
            assert_eq!(got1, data);
        }
        let data1 = vec![8, 9, 10, 11, 12, 13, 14, 15].repeat(999);
        let range1 = 0..data1.len();
        block.put(global.clone(), data1.clone(), range1.clone()).await.unwrap();
        let mut got2 = Vec::new();
        {
            let mut stream = block.get(global.clone(), range1.clone());
            while let Some(chunk) = stream.next().await {
                got2.extend(chunk.unwrap());
            }
            assert_eq!(got2, data1);
        }
        block.delete(global.clone()).await;
    }
}