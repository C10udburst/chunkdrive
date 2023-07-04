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
    let source = global.get_source(global.random_source()).unwrap();
    println!("{:?}", source);
    let block = vec![104, 101, 108, 108, 111];
    let descriptor = source.create(&block).await.unwrap();
    let data = source.get(&descriptor).await.unwrap();
    assert_eq!(block, data);
}
