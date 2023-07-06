use std::ops::Range;
use async_trait::async_trait;
use super::{direct_block::DirectBlock, indirect_block::IndirectBlock, stored_block::StoredBlock};
use serde::{Serialize, Deserialize};

use crate::{global::Global, sources::error::SourceError};

#[async_trait]
pub trait IBlock {
    async fn range(&self, global: &Global) -> Result<Range<usize>, SourceError>;
    async fn intersects(&self, range: &Range<usize>, global: &Global) -> Result<bool, SourceError>;
    async fn get(&self, global: &Global, range: &Range<usize>) -> Result<Vec<u8>, SourceError>;
    async fn replace(&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError>;
    async fn put(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>;
    async fn truncate(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>;
    async fn extend(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>;
    async fn delete(&self, global: &Global) -> Result<(), SourceError>;
    async fn heal(&mut self, global: &Global) -> Result<(), SourceError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BlockType {
    DirectBlock(DirectBlock),
    IndirectBlock(IndirectBlock),
    StoredBlock(StoredBlock),
}

impl BlockType {
    pub fn as_dyn(&self) -> &dyn IBlock {
        match self {
            BlockType::DirectBlock(block) => block,
            BlockType::IndirectBlock(block) => block,
            BlockType::StoredBlock(block) => block,
        }
    }
    pub async fn update(&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError> {
        let range = self.range(global).await?;
        let size = data.len();
        if range.len() < size {
            self.extend(global, &(0..size), data).await?;
        } else if range.len() > size {
            self.truncate(global, &(0..size), data).await?;
        } else {
            self.replace(global, data).await?;
        }
        Ok(())
    }
}

macro_rules! impl_method {
    ($method:ident, (&self, $($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl BlockType {
            pub async fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    BlockType::DirectBlock(block) => block.$method($($arg),*).await,
                    BlockType::IndirectBlock(block) => block.$method($($arg),*).await,
                    BlockType::StoredBlock(block) => block.$method($($arg),*).await,
                }
            }
        }
    };
    ($method:ident, (&mut self, $($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl BlockType {
            pub async fn $method(&mut self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    BlockType::DirectBlock(block) => block.$method($($arg),*).await,
                    BlockType::IndirectBlock(block) => block.$method($($arg),*).await,
                    BlockType::StoredBlock(block) => block.$method($($arg),*).await,
                }
            }
        }
    };
}

impl_method!(range, (&self, global: &Global) -> Result<Range<usize>, SourceError>);
impl_method!(intersects, (&self, range: &Range<usize>, global: &Global) -> Result<bool, SourceError>);
impl_method!(get, (&self, global: &Global, range: &Range<usize>) -> Result<Vec<u8>, SourceError>);
impl_method!(replace, (&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError>);
impl_method!(put, (&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>);
impl_method!(truncate, (&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>);
impl_method!(extend, (&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError>);
impl_method!(delete, (&self, global: &Global) -> Result<(), SourceError>);
impl_method!(heal, (&mut self, global: &Global) -> Result<(), SourceError>);