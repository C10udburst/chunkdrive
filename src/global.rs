use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::{inodes::folder::Folder};
use rmp_serde::{Deserializer, Serializer};

use super::source::Source;

const ROOT_PATH: &str = "root.dat";

#[derive(Deserialize, Debug)]
pub struct Global {
    sources: HashMap<String, Source>,

    #[serde(default = "default_redundancy")]
    pub redundancy: usize,
    #[serde(default = "default_max_direct_blocks")]
    pub max_direct_blocks: usize,

    #[serde(skip_deserializing)]
    #[serde(default = "default_root")]
    root: Mutex<Folder>,
}

fn default_redundancy() -> usize { 1 }
fn default_max_direct_blocks() -> usize { 10 }
fn default_root() -> Mutex<Folder> { 
    if let Ok(file) = std::fs::File::open(ROOT_PATH) {
        let mut de = Deserializer::new(&file);
        return Mutex::new(Folder::deserialize(&mut de).unwrap());
    } else {
        let root = Folder::create().unwrap();
        let file = std::fs::File::create(ROOT_PATH).unwrap();
        let mut ser = Serializer::new(file);
        root.serialize(&mut ser).unwrap();
        return Mutex::new(root);
    }
}


impl Global {
    pub fn random_source(&self) -> &String {
        let keys: Vec<_> = self.sources.keys().collect();
        let random_key = keys[rand::thread_rng().gen_range(0..keys.len())];
        random_key
    }
    
    pub fn get_source(&self, name: &str) -> Option<&Source> {
        self.sources.get(name)
    }

    pub fn get_root(&self) -> &Mutex<Folder> {
        &self.root
    }

    pub fn root_updated(&self) {
        let file = std::fs::File::create(ROOT_PATH).unwrap();
        let mut ser = Serializer::new(file);
        self.root.lock().unwrap().serialize(&mut ser).unwrap();
    }
}