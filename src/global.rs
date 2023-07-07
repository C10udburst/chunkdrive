use std::collections::HashMap;

use rand::seq::IteratorRandom;
use serde::Deserialize;

use crate::bucket::Bucket;

#[derive(Deserialize, Debug)]
pub struct Global {
    buckets: HashMap<String, Bucket>,
}

impl Global {
    pub fn get_bucket(&self, name: &str) -> Option<&Bucket> {
        self.buckets.get(name)
    }
    
    pub fn random_bucket_sized(&self, max_size: usize) -> Option<&Bucket> {
        self.buckets
            .values()
            .filter(|bucket| bucket.max_size() >= max_size)
            .choose(&mut rand::thread_rng())
    }

    pub fn random_bucket(&self) -> Option<&Bucket> {
        self.buckets.values().choose(&mut rand::thread_rng())
    }
}