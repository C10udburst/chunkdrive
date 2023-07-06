use async_trait::async_trait;
use async_recursion::async_recursion;
use serde::{Serialize, Deserialize};
use std::ops::Range;
use crate::{global::Global, sources::error::SourceError};

use super::{block::{IBlock, BlockType}, direct_block::DirectBlock, stored_block::StoredBlock};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndirectBlock {
    blocks: Vec<BlockType>
}

#[async_trait]
impl IBlock for IndirectBlock {
    async fn range(&self, global: &Global) -> Result<Range<usize>, SourceError> {
        let start = self.blocks.first().unwrap().range(global).await?.start;
        let end = self.blocks.last().unwrap().range(global).await?.end;
        Ok(start..end)
    }

    async fn intersects(&self, range: &Range<usize>, global: &Global) -> Result<bool, SourceError> {
        let self_range = self.range(global).await?;
        Ok(self_range.start <= range.start && self_range.end >= range.end)
    }

    async fn get(&self, global: &Global, range: &Range<usize>) -> Result<Vec<u8>, SourceError> {
        let mut data = Vec::new();
        for block in &self.blocks {
            if block.intersects(range, global).await? {
                data.append(&mut block.get(global, range).await?);
            }
        }
        Ok(data)
    }

    async fn replace(&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError> {
        for block in &mut self.blocks {
            let slice = block.range(global).await?;
            block.replace(global, &data[slice.clone()].to_vec()).await?;
        }
        Ok(())
    }

    async fn put(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        for block in &mut self.blocks {
            if block.intersects(range, global).await? {
                block.put(global, range, data).await?;
            }
        }
        Ok(())
    }

    async fn truncate(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut to_delete: Vec<usize> = Vec::new();
        
        let i = 0;
        for block in &mut self.blocks {
            if block.intersects(range, global).await? {
                block.truncate(global, range, data).await?;
            }
            if block.range(global).await?.start > range.end {
                to_delete.push(i);
            }
        }
        
        for block in to_delete {
            match self.blocks.remove(block).delete(global).await {
                Ok(_) => {},
                Err(_) => {}
            }
        }
        
        Ok(())
    }

    async fn extend(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut start = 0;
        for block in &mut self.blocks {
            if block.intersects(range, global).await? {
                block.extend(global, range, data).await?;
            }
            start = block.range(global).await?.end + 1;
        }

        let end = range.end;
        while start < end && self.blocks.len() < global.max_direct_blocks {
            let (block, block_end) = DirectBlock::create(global, &(start..end), &data[start..end].to_vec()).await?;
            self.blocks.push(block.to_enum());
            start = block_end + 1;
        }
        if start < end {
            let block = IndirectBlock::create(global, &data[start..end].to_vec()).await?;
            let stored = StoredBlock::create(global, block.to_enum()).await?;
            self.blocks.push(stored.to_enum());
        }
              
        Ok(())
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        for block in &self.blocks {
            block.delete(global).await?;
        }
        Ok(())
    }

    async fn heal(&mut self, global: &Global) -> Result<(), SourceError> {
        for block in &mut self.blocks {
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
    pub async fn create(global: &Global, data: &Vec<u8>) -> Result<IndirectBlock, SourceError> {
        let mut blocks = Vec::new();
        let mut start = 0;
        let end = data.len();
        let direct = 0;
        while start < end && direct < global.max_direct_blocks {
            let (block, block_end) = DirectBlock::create(global, &(start..end), &data[start..end].to_vec()).await?;
            blocks.push(block.to_enum());
            start = block_end + 1;
        }
        if start < end {
            let block = IndirectBlock::create(global, &data[start..end].to_vec()).await?;
            let stored = StoredBlock::create(global, block.to_enum()).await?;
            blocks.push(stored.to_enum());
        }
        Ok(IndirectBlock {
            blocks
        })
    }
}