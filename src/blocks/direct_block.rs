/*
    This block stores the data directly in the buckets.
    It does not split the data into chunks.
 */

use std::{ops::Range, sync::Arc};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Serialize, Deserialize};

use crate::global::{Global, Descriptor};
use super::block::{Block, BlockType};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectBlock {
    #[serde(rename = "b")]
    bucket: String,
    #[serde(rename = "d")]
    descriptor: Descriptor,
    #[serde(rename = "r")]
    range: Range<usize>,
} // we use short names to reduce the size of the serialized data while allowing backwards compatibility

#[async_trait]
impl Block for DirectBlock {
    async fn range(&self, _global: Arc<Global>) -> Result<Range<usize>, String> {
        Ok(self.range.clone())
    }

    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            if range.end <= self.range.start || range.start >= self.range.end {
                return // the range is outside of the block, so we return an empty stream
            }
            let bucket = match global.get_bucket(&self.bucket) {
                Some(bucket) => bucket,
                None => Err("Bucket not found".to_string())?
            };
            let data = match bucket.get(&self.descriptor).await {
                Ok(data) => data,
                Err(_) => Err("Could not get the data".to_string())?
            };

            // calculate the data slice
            let start = std::cmp::max(range.start, self.range.start) - self.range.start;
            let end = std::cmp::min(range.end, self.range.end) - self.range.start;
            let data = data[start..end].to_vec();
            yield Ok(data);
        })
    }

    // indirect blocks ensure that the data.length == range.length && data[0] corresponds to range.start
    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, _range: Range<usize>) -> Result<(), String> {
        // put the data
        let bucket = match global.get_bucket(&self.bucket) {
            Some(bucket) => bucket,
            None => return Err("Bucket not found".to_string())
        };
        match bucket.put(&self.descriptor, data.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Could not put the data: {}", e))
        }
    }

    async fn delete(&self, global: Arc<Global>) -> Result<(), String> {
        let bucket = match global.get_bucket(&self.bucket) {
            Some(bucket) => bucket,
            None => return Err("Bucket not found".to_string())
        };
        bucket.delete(&self.descriptor).await
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        // finding the buckets
        let bucket_name = global.random_bucket().ok_or("No buckets found".to_string())?;
        let bucket = match global.get_bucket(bucket_name) {
            Some(bucket) => bucket,
            None => Err("Bucket not found".to_string())?
        };

        // slice the data
        let data = data[..std::cmp::min(data.len(), bucket.max_size())].to_vec();
        if data.is_empty() {
            return Err("Data is empty".to_string())
        }

        // create descriptors
        let bucket = match global.get_bucket(bucket_name) {
            Some(bucket) => bucket,
            None => Err("Bucket not found".to_string())?
        };
        let descriptor = match bucket.create().await {
            Ok(descriptor) => descriptor,
            Err(e) => return Err(format!("Could not create the descriptor: {}", e))
        };

        // put the data
        let bucket = match global.get_bucket(bucket_name) {
            Some(bucket) => bucket,
            None => Err("Bucket not found".to_string())?
        };
        bucket.put(&descriptor, data.clone()).await?;

        Ok(BlockType::Direct(DirectBlock {
            range: start..start + data.len(),
            bucket: bucket_name.clone(),
            descriptor
        }))
    }

    fn to_enum(self) -> BlockType {
        BlockType::Direct(self)
    }
}