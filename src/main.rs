use block::direct_block::DirectBlock;
use serde_yaml::from_reader;
use std::fs::File;

mod global;
mod source;
mod sources;
mod encryption;
mod block;

use global::Global;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let file = File::open("config.yml").unwrap();
    let global: Global = from_reader(file).unwrap();
    let data: Vec<u8> = "Hello, world!".as_bytes().to_vec();
    
}
