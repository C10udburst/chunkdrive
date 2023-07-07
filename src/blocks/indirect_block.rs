use std::{ops::Range, sync::Arc};

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use serde::{Serialize, Deserialize};

use crate::global::Global;
use super::{block::{Block, BlockType}, direct_block::DirectBlock, stored_block::StoredBlock};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndirectBlock {
    #[serde(rename = "b")]
    blocks: Vec<BlockType>,  // we will make sure that these are in order
}

#[async_trait]
impl Block for IndirectBlock {
    async fn range(&self, global: Arc<Global>) -> Result<Range<usize>, String> {
        let first = match self.blocks.first() {
            Some(block) => block,
            None => return Ok(0..0),
        };
        let last = match self.blocks.last() {
            Some(block) => block,
            None => panic!("This should never happen"),
        };
        let first_range = first.range(global.clone()).await?;
        let last_range = last.range(global.clone()).await?;
        Ok(first_range.start..last_range.end)
    }

    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            for block in self.blocks.iter() {
                let global_clone = global.clone();
                let range_clone = range.clone();
                let mut stream = block.get(global_clone, range_clone);
                while let Some(data) = stream.next().await {
                    yield data;
                }
            }
        })
    }

    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, range: Range<usize>) -> Result<(), String> {
        let mut start = range.start;
        
        for block in self.blocks.iter_mut() {
            let block_range = block.range(global.clone()).await?;
            if block_range.end <= start { break; }
            let offet_range = start..(start + data.len());
            let slice = data.get(offet_range.clone()).unwrap().to_vec();
            block.put(global.clone(), slice, block_range).await?;
        }

        // if data is left, we create new blocks just like we did in the create function
        Ok(())
    }

    async fn delete(&self, global: Arc<Global>) {
        for block in self.blocks.iter() {
            block.delete(global.clone()).await;
        }
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        let mut blocks = Vec::new();
        let mut start = start;
        let end = start + data.len();
        while start < end && blocks.len() < global.direct_block_count {
            let block = DirectBlock::create(global.clone(), data.clone(), start).await?;
            let range = block.range(global.clone()).await?;
            start = range.end;
        }
        // if there is still data left, we create a stored block
        blocks.push(StoredBlock::create(global.clone(), data, start).await?);
        Ok(BlockType::Indirect(IndirectBlock {
            blocks,
        }))
    }

    async fn repair(&self, global: Arc<Global>, range: Range<usize>) -> Result<(), String> {
        for block in self.blocks.iter() {
            block.repair(global.clone(), range.clone()).await?;
        }
        Ok(())
    }

    fn into(self) -> BlockType {
        BlockType::Indirect(self)
    }

}