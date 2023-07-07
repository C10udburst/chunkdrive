use std::collections::HashMap;

use rand::seq::IteratorRandom;
use serde::Deserialize;

use crate::bucket::Bucket;

#[derive(Deserialize, Debug)]
pub struct Global {
    buckets: HashMap<String, Bucket>,

    #[serde(default = "default_redundancy")]
    pub redundancy: usize,
    #[serde(default = "default_direct_block_count")]
    pub direct_block_count: usize,
}

const fn default_redundancy() -> usize { 1 } // disabled by default
const fn default_direct_block_count() -> usize { 10 }

impl Global {
    pub fn get_bucket(&self, name: &str) -> Option<&Bucket> {
        self.buckets.get(name)
    }
    
    pub fn random_bucket_sized(&self, max_size: usize) -> Option<&String> {
        self.buckets
            .iter()
            .filter(|(_, bucket)| bucket.max_size() >= max_size)
            .choose(&mut rand::thread_rng())
            .map(|(bucket, _)| bucket)
    }

    pub fn random_bucket(&self) -> Option<&String> {
        self.buckets
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|(bucket, _)| bucket)
    }
}