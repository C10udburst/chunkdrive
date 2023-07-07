use std::collections::HashMap;

use serde::Deserialize;

use crate::bucket::Bucket;

#[derive(Deserialize, Debug)]
pub struct Global {
    pub buckets: HashMap<String, Bucket>,
}