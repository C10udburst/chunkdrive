use async_trait::async_trait;
use async_recursion::async_recursion;
use serde::{Serialize, Deserialize};
use std::ops::Range;
use crate::{global::Global, sources::error::SourceError};

use super::{block::{IBlock, BlockType}, direct_block::DirectBlock};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndirectBlock {
    blocks: Vec<BlockType>
}

#[async_trait]
impl IBlock for IndirectBlock {
    fn range(&self) -> Range<usize> {
        let start = self.blocks.first().unwrap().range().start;
        let end = self.blocks.last().unwrap().range().end;
        start..end
    }

    fn intersects(&self, range: Range<usize>) -> bool {
        let self_range = self.range();
        self_range.start <= range.start && self_range.end >= range.end
    }

    async fn get(&self, global: &Global, range: Range<usize>) -> Result<Vec<u8>, SourceError> {
        let mut data = Vec::new();
        for block in &self.blocks {
            if block.intersects(range.clone()) {
                data.append(&mut block.get(global, range.clone()).await?);
            }
        }
        Ok(data)
    }

    async fn replace(&mut self, global: &Global, data: Vec<u8>) -> Result<(), SourceError> {
        let mut start = 0;
        let end = data.len();
        for block in &mut self.blocks {
            block.replace(global, data.clone()[start..end].to_vec()).await?;
            start += block.range().len();
        }
        Ok(())
    }

    async fn put(&mut self, global: &Global, range: Range<usize>, data: Vec<u8>) -> Result<(), SourceError> {
        for block in &mut self.blocks {
            if block.intersects(range.clone()) {
                block.put(global, range.clone(), data.clone()).await?;
            }
        }
        Ok(())
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        for block in &self.blocks {
            block.delete(global).await?;
        }
        Ok(())
    }

    async fn heal(&self, global: &Global) -> Result<(), SourceError> {
        for block in &self.blocks {
            block.heal(global).await?;
        }
        Ok(())
    }
}

impl IndirectBlock {
    pub fn to_enum(self) -> BlockType {
        BlockType::IndirectBlock(self)
    }

    #[async_recursion]
    pub async fn create(global: &Global, data: Vec<u8>) -> Result<IndirectBlock, SourceError> {
        let mut blocks = Vec::new();
        let mut start = 0;
        let end = data.len();
        let direct = 0;
        while start < end && direct < global.max_direct_blocks {
            let (block, block_end) = DirectBlock::create(global, start..end, &data[start..end].to_vec()).await?;
            blocks.push(block.to_enum());
            start = block_end + 1;
        }
        blocks.push(IndirectBlock::create(global, data[start..end].to_vec()).await?.to_enum());
        Ok(IndirectBlock {
            blocks
        })
    }
}