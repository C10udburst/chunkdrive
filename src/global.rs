use std::collections::HashMap;
use rand::Rng;

use serde::Deserialize;

use super::source::Source;

#[derive(Deserialize, Debug)]
pub struct Global {
    sources: HashMap<String, Source>,

    #[serde(default = "default_redundancy")]
    pub redundancy: usize,
}

fn default_redundancy() -> usize { 1 }

impl Global {
    pub fn random_source(&self) -> &String {
        let keys: Vec<_> = self.sources.keys().collect();
        let random_key = keys[rand::thread_rng().gen_range(0..keys.len())];
        random_key
    }
    
    pub fn get_source(&self, name: &str) -> Option<&Source> {
        self.sources.get(name)
    }
}