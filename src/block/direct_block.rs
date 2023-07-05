use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::ops::Range;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::{global::Global, sources::error::SourceError};


use super::block::{IBlock, BlockType};

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectBlock {
    range: Range<usize>,
    sources: Vec<(String, Vec<u8>)>,
    sha1: Vec<u8>,
}

async fn load_from_source(name: &str, descriptor: &[u8], global: &Global) -> Result<Vec<u8>, SourceError> {
    let source_impl = match global.get_source(&name) {
        Some(source) => source,
        None => return Err(SourceError::new(format!("Source {} not found", &name)))
    };
    let data = source_impl.get(descriptor).await?;
    Ok(data)
}

async fn put_to_source(name: &str, descriptor: &[u8], data: Vec<u8>, global: &Global) -> Result<(), SourceError> {
    let source_impl = match global.get_source(&name) {
        Some(source) => source,
        None => return Err(SourceError::new(format!("Source {} not found", &name)))
    };
    source_impl.put(descriptor, &data).await?;
    Ok(())
}

#[async_trait]
impl IBlock for DirectBlock {
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
            let start = if range.start < self.range.start { 0 } else { range.start - self.range.start };
            let end = if range.end > self.range.end { source_data.len() } else { range.end - self.range.start };
            data.extend(source_data.drain(start..end));
            let mut sha = Sha1::new();
            sha.input(&data);
            let mut result = vec![0; sha.output_bytes()];
            sha.result(&mut result);
            if result != self.sha1 {
                // TODO: schedule heal
                data.clear();
            } else { break; }
        }
        Ok(data)
    }

    async fn replace(&mut self, global: &Global, data: Vec<u8>) -> Result<(), SourceError> {
        for (name, descriptor) in &self.sources {
            put_to_source(name, descriptor, data.clone(), global).await?;
        }
        let mut sha = Sha1::new();
        sha.input(&data);
        let mut result = vec![0; sha.output_bytes()];
        sha.result(&mut result);
        self.sha1.clear();
        self.sha1.extend(result);
        Ok(())
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        for (name, descriptor) in &self.sources {
            let source_impl = match global.get_source(&name) {
                Some(source) => source,
                None => continue
            };
            match source_impl.delete(descriptor).await {
                Ok(_) => continue,
                Err(_) => continue
            }
        }
        Ok(())
    }

    async fn heal(&self, global: &Global) -> Result<(), SourceError> {
        Ok(())
    }
}

impl DirectBlock {
    pub async fn new(global: &Global, range: Range<usize>, data: &Vec<u8>) -> Result<DirectBlock, SourceError> {
        let mut sources: Vec<(String, Vec<u8>)> = Vec::new();
        let mut redundancy = global.redundancy;
        while redundancy > 0 {
            let name = global.random_source();
            if sources.iter().any(|(name, _)| name.eq(name)) {
                continue;
            }
            let source = match global.get_source(&name) {
                Some(source) => source,
                None => continue
            };
            let descriptor = match source.create(data).await {
                Ok(descriptor) => descriptor,
                Err(_) => continue
            };
            sources.push((name.clone(), descriptor));
            redundancy -= 1;
        }

        let mut sha = Sha1::new();
        sha.input(data);
        let mut result = vec![0; sha.output_bytes()];
        sha.result(&mut result);

        Ok(DirectBlock {
            range,
            sources,
            sha1: result,
        })
    }

    pub fn to_enum(self) -> BlockType {
        BlockType::DirectBlock(self)
    }
}