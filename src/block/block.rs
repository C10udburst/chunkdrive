use std::ops::Range;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{global::Global, sources::error::SourceError};

#[async_trait]
pub trait IBlock {
    fn range(&self) -> &Range<usize>;
    fn intersects(&self, range: Range<usize>) -> bool;
    async fn get(&self, global: &Global, range: Range<usize>) -> Result<Vec<u8>, SourceError>;
    async fn replace(&self, global: &Global, data: Vec<u8>) -> Result<(), SourceError>;
    async fn delete(&self, global: &Global) -> Result<(), SourceError>;
}

impl dyn IBlock {
    async fn put(&self, global: Global, range: Range<usize>, data: Vec<u8>) -> Result<(), SourceError> {
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