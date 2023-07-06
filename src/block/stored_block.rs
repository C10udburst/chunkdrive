use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::ops::Range;
use crate::{global::Global, sources::error::SourceError, stored::Stored};

use super::{block::{IBlock, BlockType}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredBlock {
    block: Stored
}

#[async_trait]
impl IBlock for StoredBlock {
    async fn range(&self, global: &Global) -> Result<Range<usize>, SourceError> {
        let block = self.block.get::<BlockType>(global).await?;
        block.range(global).await
    }

    async fn intersects(&self, range: &Range<usize>, global: &Global) -> Result<bool, SourceError> {
        let block = self.block.get::<BlockType>(global).await?;
        block.intersects(range, global).await
    }

    async fn get(&self, global: &Global, range: &Range<usize>) -> Result<Vec<u8>, SourceError> {
        let block = self.block.get::<BlockType>(global).await?;
        block.get(global, range).await
    }

    async fn replace(&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut block = self.block.get::<BlockType>(global).await?;
        block.replace(global, data).await?;
        self.block.put(global, &block).await?;
        Ok(())
    }

    async fn put(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut block = self.block.get::<BlockType>(global).await?;
        block.put(global, range, data).await?;
        self.block.put(global, &block).await?;
        Ok(())
    }

    async fn truncate(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut block = self.block.get::<BlockType>(global).await?;
        block.truncate(global, range, data).await?;
        self.block.put(global, &block).await?;
        Ok(())
    }

    async fn extend(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let mut block = self.block.get::<BlockType>(global).await?;
        block.extend(global, range, data).await?;
        self.block.put(global, &block).await?;
        Ok(())
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        let block = self.block.get::<BlockType>(global).await?;
        block.delete(global).await?;
        self.block.delete(global).await?;
        Ok(())
    }

    async fn heal(&mut self, global: &Global) -> Result<(), SourceError> {
        let mut block = self.block.get::<BlockType>(global).await?;
        block.heal(global).await?;
        self.block.put(global, &block).await?;
        Ok(())
    }
}

impl StoredBlock {
    pub async fn create(global: &Global, block: BlockType) -> Result<Self, SourceError> {
        let block = Stored::new(global, &block).await?;
        Ok(Self {
            block
        })
    }

    pub fn to_enum(self) -> BlockType {
        BlockType::StoredBlock(self)
    }
}