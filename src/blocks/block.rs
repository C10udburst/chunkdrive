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
    async fn delete(&self, global: Arc<Global>) -> Result<(), String>;
    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String>;
    fn to_enum(self) -> BlockType;
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

    async fn delete(&self, global: Arc<Global>) -> Result<(), String> {
        match_method!(self, delete, global).await
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        IndirectBlock::create(global, data, start).await // we use indirect blocks, because they will fit any data size
    }

    fn to_enum(self) -> BlockType {
        self
    }
}
