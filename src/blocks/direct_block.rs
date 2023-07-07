/*
    This block stores the data directly in the buckets.
    It does not split the data into chunks.
 */

use std::{ops::Range, sync::Arc, vec};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Serialize, Deserialize};
use crypto::{
    md5::Md5,
    digest::Digest
};

use crate::global::Global;
use super::block::{Block, BlockType};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectBlock {
    // each entry is a bucket name and a descriptor
    #[serde(rename = "s")]
    sources: Vec<(String, String)>,
    #[serde(rename = "r")]
    range: Range<usize>,
    // the md5 hash of the data
    #[serde(rename = "h")]
    hash: Vec<u8>,
} // we use short names to reduce the size of the serialized data while allowing backwards compatibility

#[async_trait]
impl Block for DirectBlock {
    async fn range(&self, _global: Arc<Global>) -> Result<Range<usize>, String> {
        Ok(self.range.clone())
    }

    fn get(&self, global: Arc<Global>, range: Range<usize>) -> BoxStream<Result<Vec<u8>, String>> {
        Box::pin(async_stream::stream! {
            let mut had_error = false;
            let mut found = false;
            let mut md5 = Md5::new();
            for (bucket, descriptor) in self.sources.iter() {
                let bucket = match global.get_bucket(bucket) {
                    Some(bucket) => bucket,
                    None => {
                        had_error = true;
                        continue
                    }
                };
                let data = match bucket.get(descriptor).await {
                    Ok(data) => data,
                    Err(_) => {
                        had_error = true;
                        continue
                    }
                };
                md5.reset();
                let mut hash = vec![0; 16];
                md5.input(&data);
                md5.result(&mut hash);
                if hash != self.hash {
                    had_error = true;
                    continue
                }
                found = true;

                // if the range is the same as the data, we can yield the data directly
                if range.len() == data.len() {
                    yield Ok(data);
                } else {
                    yield Ok(data[0..range.len()].to_vec());
                }
                
            }
            if !found {
                yield Err("Could not find the data".to_string());
            }
            if had_error {
                // TODO: schedule a repair
            }
        })
    }

    // indirect blocks ensure that the data.length == range.length && data[0] corresponds to range.start
    async fn put(&mut self, global: Arc<Global>, data: Vec<u8>, _range: Range<usize>) -> Result<(), String> {
        // put the data
        let mut failed_count = 0;
        for (bucket_name, descriptor) in self.sources.iter() {
            let bucket = match global.get_bucket(bucket_name) {
                Some(bucket) => bucket,
                None => {
                    failed_count += 1;
                    continue
                }
            };
            match bucket.put(descriptor, data.clone()).await {
                Ok(_) => continue,
                Err(_) => {
                    failed_count += 1;
                    continue
                }
            }
        }

        if failed_count == self.sources.len() {
            return Err("Could not put the data".to_string())
        }

        // hash
        let mut hash = vec![0; 16];
        let mut md5 = Md5::new();
        md5.input(&data);
        md5.result(&mut hash);
        self.hash = hash;

        if failed_count > 0 {
            // TODO: schedule a repair
        }

        Ok(())
    }

    async fn delete(&self, global: Arc<Global>) {
        for (bucket, descriptor) in self.sources.iter() {
            let bucket = match global.get_bucket(bucket) {
                Some(bucket) => bucket,
                None => continue
            };
            let _ = bucket.delete(descriptor).await;
        }
    }

    async fn create(global: Arc<Global>, data: Vec<u8>, start: usize) -> Result<BlockType, String> {
        // finding the buckets
        let mut buckets = vec![global.random_bucket().ok_or("No buckets available".to_string())?];
        let base_bucket = match global.get_bucket(&buckets[0]) {
            Some(bucket) => bucket,
            None => return Err("No buckets available".to_string())
        };
        for _ in 1..global.redundancy {
            let bucket = global.next_bucket(base_bucket.max_size(), &buckets).ok_or(format!("Not enough buckets available ({} needed)", global.redundancy))?;
            buckets.push(bucket);
        }

        // slice the data
        let data = data[..base_bucket.max_size()].to_vec();
        if data.len() == 0 {
            return Err("Data is empty".to_string())
        }

        // hashing the data
        let mut hash = vec![0; 16];
        let mut md5 = Md5::new();
        md5.input(&data);
        md5.result(&mut hash);

        // create descriptors
        let mut failed = false;
        let mut descriptors = Vec::new();
        for bucket_name in buckets.iter() {
            let bucket = match global.get_bucket(bucket_name) {
                Some(bucket) => bucket,
                None => {
                    failed = true;
                    break
                }
            };
            let descriptor = match bucket.create().await {
                Ok(descriptor) => descriptor,
                Err(_) => {
                    failed = true;
                    break
                }
            };
            descriptors.push((bucket_name.to_owned().to_owned(), descriptor));
        }
        
        if failed {
            // remove the descriptors if they were created
            for (bucket_name, descriptor) in descriptors.iter() {
                let bucket = match global.get_bucket(bucket_name) {
                    Some(bucket) => bucket,
                    None => continue
                };
                let _ = bucket.delete(descriptor).await;
            }
        }

        // put the data
        let mut failed_descriptors = Vec::new();
        for (bucket_name, descriptor) in descriptors.iter() {
            let bucket = match global.get_bucket(bucket_name) {
                Some(bucket) => bucket,
                None => {
                    failed_descriptors.push((bucket_name.to_owned(), descriptor.to_owned()));
                    continue
                }
            };
            match bucket.put(descriptor, data.clone()).await {
                Ok(_) => continue,
                Err(_) => {
                    failed_descriptors.push((bucket_name.to_owned(), descriptor.to_owned()));
                    continue
                }
            }
        }

        // remove failed descriptors
        for (bucket_name, descriptor) in failed_descriptors.iter() {
            let bucket = match global.get_bucket(bucket_name) {
                Some(bucket) => bucket,
                None => continue
            };
            let _ = bucket.delete(descriptor).await;
        }
        descriptors.retain(|(bucket_name, descriptor)| !failed_descriptors.contains(&(bucket_name.to_owned(), descriptor.to_owned())));

        Ok(BlockType::Direct(DirectBlock {
            sources: descriptors,
            range: start..start + base_bucket.max_size(),
            hash,
        }))
    }

    async fn repair(&self, _global: Arc<Global>, _range: Range<usize>) -> Result<(), String> {
        Err("Not implemented".to_string())
    }

    fn into(self) -> BlockType {
        BlockType::Direct(self)
    }
}