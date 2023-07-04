use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::{ops::Range, vec};
use crate::{global::Global, sources::error::SourceError};

use super::block::IBlock;

#[derive(Serialize, Deserialize, Debug)]
pub struct Stored {
    range: Range<usize>,
    sources: Vec<(String, Vec<u8>)>
}

async fn load_from_source(name: &str, descriptor: &[u8], global: &Global) -> Result<Vec<u8>, SourceError> {
    let source_impl = match global.get_source(&name) {
        Some(source) => source,
        None => return Err(SourceError::new(format!("Source {} not found", &name)))
    };
    let data = source_impl.get(descriptor).await?;
    Ok(data)
}

#[async_trait]
impl IBlock for Stored {
    fn range(&self) -> &Range<usize> {
        &self.range
    }

    fn intersects(&self, range: Range<usize>) -> bool {
        self.range.start < range.end && self.range.end > range.start
    }

    async fn get(&self, global: &Global, range: Range<usize>) -> Result<Vec<u8>, SourceError> {
        let mut data = Vec::new();
        for (name, descriptor) in &self.sources {
            let mut source_data = load_from_source(name, descriptor, global).await?;
            let start = match range.start - self.range.start {
                x if x < 0 => 0,
                x => x
            };
            let end = match range.end - self.range.start {
                x if x > self.range.end => self.range.end,
                x => x
            };
            data.extend_from_slice(&source_data[start..end]);
        }
        Ok(data)
    }

    async fn replace(&self, global: &Global, data: Vec<u8>) -> Result<(), SourceError> {
        unimplemented!()
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        unimplemented!()
    }   

}