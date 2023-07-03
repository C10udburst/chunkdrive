use std::collections::HashMap;
use rand::Rng;

use serde::Deserialize;

use crate::sources::source::{SourceType, Source};

#[derive(Deserialize, Debug)]
pub struct Global {
    sources: HashMap<String, SourceType>,
}

impl Global {
    pub fn random_source(&self) -> &String {
        let mut rng = rand::thread_rng();
        let keys: Vec<_> = self.sources.keys().collect();
        let random_key = keys[rand::thread_rng().gen_range(0..keys.len())];
        random_key
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Source> {
        self.sources.get(name).and_then(|source| source.get())
    }
}