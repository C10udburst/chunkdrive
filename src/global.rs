use std::collections::HashMap;

use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use rmp_serde::{Deserializer, Serializer};

use crate::{bucket::Bucket, inodes::directory::Directory, stored::Stored};

#[derive(Deserialize, Debug)]
pub struct Global {
    buckets: HashMap<String, Bucket>,

    #[serde(default = "default_redundancy")]
    pub redundancy: usize,
    #[serde(default = "default_direct_block_count")]
    pub direct_block_count: usize,
    
    #[serde(default = "default_root_path")]
    root_path: String,
}

const fn default_redundancy() -> usize { 1 } // redundancy is disabled by default, so we set it to 1
const fn default_direct_block_count() -> usize { 10 }
fn default_root_path() -> String { "./root.dat".to_string() }

impl Global {
    pub fn get_bucket(&self, name: &str) -> Option<&Bucket> {
        self.buckets.get(name)
    }
    
    pub fn next_bucket(&self, max_size: usize, exclude: &Vec<&String>) -> Option<&String> {
        self.buckets
            .iter()
            .filter(|(_, bucket)| bucket.max_size() >= max_size)
            .filter(|(bucket, _)| !exclude.contains(bucket))
            .choose(&mut rand::thread_rng())
            .map(|(bucket, _)| bucket)
    }

    pub fn random_bucket(&self) -> Option<&String> {
        self.buckets
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|(bucket, _)| bucket)
    }
    
    pub fn get_root(&self) -> Directory {
        match std::fs::File::open(&self.root_path) {
            Ok(file) => {
                let mut de = Deserializer::new(&file);
                match Deserialize::deserialize(&mut de) {
                    Ok(root) => root,
                    Err(_) => {
                        std::fs::remove_file(&self.root_path).unwrap();
                        Directory::new()
                    }
                }
            },
            Err(_) => {
                Directory::new()
            }
        }
    }

    pub fn save_root(&self, root: &Directory) {
        let mut file = std::fs::File::create(&self.root_path).unwrap();
        root.serialize(&mut Serializer::new(&mut file)).unwrap();
    }
}