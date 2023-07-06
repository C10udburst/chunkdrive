use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::ops::Range;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use crate::{global::Global, sources::error::SourceError, source::Source};


use super::block::{IBlock, BlockType};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    async fn range(&self, _global: &Global) -> Result<Range<usize>, SourceError> {
        Ok(self.range.to_owned())
    }

    async fn intersects(&self, range: &Range<usize>, _global: &Global) -> Result<bool, SourceError> {
        Ok(self.range.start <= range.start && self.range.end >= range.end)
    }

    async fn get(&self, global: &Global, range: &Range<usize>) -> Result<Vec<u8>, SourceError> {
        let mut data = Vec::new();
        for (name, descriptor) in &self.sources {
            let mut source_data = load_from_source(name, descriptor, global).await?;
            let start = if range.start < self.range.start { 0 } else { range.start - self.range.start };
            let end = if range.end > self.range.end { source_data.len() } else { range.end - self.range.start };
            data.extend(source_data.drain(start..end));
            let mut sha: Sha1 = Sha1::new();
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

    async fn replace(&mut self, global: &Global, data: &Vec<u8>) -> Result<(), SourceError> {
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

    async fn put(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let old_data = self.get(&global, &self.range).await?;
        let mut new_data = Vec::new();
        let start = range.start - self.range.start;
        let end = range.end - self.range.end;
        new_data.extend_from_slice(&old_data[..start]);
        new_data.extend_from_slice(&data);
        new_data.extend_from_slice(&old_data[end..]);
        self.replace(&global, &new_data).await
    }

    async fn truncate(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        let old_data = self.get(&global, &self.range).await?;
        let mut new_data = Vec::new();
        let start = range.start - self.range.start;
        let end = range.end - self.range.end;
        new_data.extend_from_slice(&old_data[..start]);
        new_data.extend_from_slice(&data);
        new_data.extend_from_slice(&old_data[end..]);
        self.replace(&global, &new_data).await
    }

    async fn extend(&mut self, global: &Global, range: &Range<usize>, data: &Vec<u8>) -> Result<(), SourceError> {
        self.put(&global, &range, &data).await
    }

    async fn heal(&mut self, global: &Global) -> Result<(), SourceError> {
        let data = self.get(global, &self.range).await?;
        let mut sha = Sha1::new();
        for (name, descriptor) in &self.sources {
            let source_data = load_from_source(name, descriptor, global).await?;
            sha.input(&source_data);
            let mut result = vec![0; sha.output_bytes()];
            sha.result(&mut result);
            if result != self.sha1 {
                put_to_source(name, descriptor, data.clone(), global).await?;
            }
        }
        Ok(())
    }
}

impl DirectBlock {
    pub fn to_enum(self) -> BlockType {
        BlockType::DirectBlock(self)
    }

    pub async fn create(
        global: &Global,
        range: &Range<usize>,
        data: &Vec<u8>,
    ) -> Result<(DirectBlock, usize), SourceError> {
        // TODO: redundancy
        let mut source_name = global.random_source();
        let mut source: &Source;
        let descriptor: Vec<u8>;
        let mut block: Vec<u8>;
        loop {
            source = match global.get_source(&source_name) {
                Some(source) => source,
                None => {
                    source_name = global.random_source();
                    continue;
                }
            };
            block = data.iter().take(source.max_size()).map(|x| *x).collect();
            descriptor = match source.create(&block).await {
                Ok(descriptor) => descriptor,
                Err(_) => {
                    source_name = global.random_source();
                    continue;
                }
            };
            break
        }
        let mut sha = Sha1::new();
        sha.input(&block);
        let mut hash = vec![0; sha.output_bytes()];
        sha.result(&mut hash);
        let direct_block = DirectBlock {
            range: range.to_owned(),
            sources: vec![(source_name.clone(), descriptor)],
            sha1: hash,
        };
        Ok((direct_block, source.max_size()))

    }
}