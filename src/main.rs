use serde_yaml::from_reader;
use std::fs::File;

mod global;
use global::Global;

mod sources;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let file = File::open("config.yml").unwrap();
    let global: Global = from_reader(file).unwrap();
    let source = global.get(global.random_source()).unwrap();
    let block = vec![1, 2, 3, 4, 5];
    // create a descriptor
    let descriptor = source.create().await.unwrap();
    // put the block into the source
    source.put(&descriptor, &block).await.unwrap();
    // get the block from the source
    let block2 = source.get(&descriptor).await.unwrap();
    // check if the blocks are the same
    assert_eq!(block, block2);
}
