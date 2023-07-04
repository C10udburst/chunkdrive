use std::collections::HashMap;
use rand::Rng;

use serde::Deserialize;

use super::source::Source;

#[derive(Deserialize, Debug)]
pub struct Global {
    sources: HashMap<String, Source>,
}

impl Global {
    pub fn random_source(&self) -> &String {
        let keys: Vec<_> = self.sources.keys().collect();
        let random_key = keys[rand::thread_rng().gen_range(0..keys.len())];
        random_key
    }
    
    pub fn get(&self, name: &str) -> Option<&Source> {
        self.sources.get(name)
    }
}