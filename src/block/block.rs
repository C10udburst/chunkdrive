use std::ops::Range;
use async_trait::async_trait;
use super::direct_block::DirectBlock;
use serde::{Serialize, Deserialize};

use crate::{global::Global, sources::error::SourceError};

#[async_trait]
pub trait IBlock {
    fn range(&self) -> &Range<usize>;
    fn intersects(&self, range: Range<usize>) -> bool;
    async fn get(&self, global: &Global, range: Range<usize>) -> Result<Vec<u8>, SourceError>;
    async fn replace(&mut self, global: &Global, data: Vec<u8>) -> Result<(), SourceError>;
    async fn delete(&self, global: &Global) -> Result<(), SourceError>;
    async fn heal(&self, global: &Global) -> Result<(), SourceError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockType {
    DirectBlock(DirectBlock)
}

impl BlockType {
    pub fn as_dyn(&self) -> &dyn IBlock {
        match self {
            BlockType::DirectBlock(block) => block,
        }
    }
}

macro_rules! impl_method {
    ($method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl BlockType {
            pub async fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    BlockType::DirectBlock(block) => block.$method($($arg),*).await,
                }
            }
        }
    };
    (mut $method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl BlockType {
            pub async fn $method(&mut self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    BlockType::DirectBlock(block) => block.$method($($arg),*).await,
                }
            }
        }
    };
    (sync $method:ident, ($($arg:ident: $arg_type:ty),*) -> $return_type:ty) => {
        impl BlockType {
            pub fn $method(&self, $($arg: $arg_type),*) -> $return_type {
                match self {
                    BlockType::DirectBlock(block) => block.$method($($arg),*),
                }
            }
        }
    };
}

impl_method!(sync range, () -> &Range<usize>);
impl_method!(sync intersects, (range: Range<usize>) -> bool);
impl_method!(get, (global: &Global, range: Range<usize>) -> Result<Vec<u8>, SourceError>);
impl_method!(mut replace, (global: &Global, data: Vec<u8>) -> Result<(), SourceError>);
impl_method!(delete, (global: &Global) -> Result<(), SourceError>);
impl_method!(heal, (global: &Global) -> Result<(), SourceError>);

impl BlockType {
    pub async fn put(&mut self, global: Global, range: Range<usize>, data: Vec<u8>) -> Result<(), SourceError> {
        let old_data = self.get(&global, self.range().clone()).await?;
        let mut new_data = Vec::new();
        let start = range.start - self.range().start;
        let end = range.end - self.range().start;
        new_data.extend_from_slice(&old_data[..start]);
        new_data.extend_from_slice(&data);
        new_data.extend_from_slice(&old_data[end..]);
        self.replace(&global, new_data).await
    }
}