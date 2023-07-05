use block::{indirect_block::IndirectBlock};
use serde_yaml::from_reader;
use std::fs::File;

mod global;
mod source;
mod sources;
mod encryption;
mod block;
mod stored;
mod inodes;

use global::Global;

use crate::{stored::Stored, block::block::BlockType};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let file = File::open("config.yml").unwrap();
    let global: Global = from_reader(file).unwrap();
    let data: Vec<u8> = "Hello, world!".as_bytes().to_vec();
    let block = IndirectBlock::create(&global, &data).await.unwrap();
    println!("{:?}", block);
    let stored = Stored::new(&global, &block).await.unwrap();
    println!("{:?}", stored);
}


#[test]
fn stored_single_block() {
    use std::env;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    let config = format!(r#"
sources:
  local1:
    source:
      !Local
      folder: {}
    encryption:
      !Aes
      key: "12345678901234567890123456789012"
      iv: "1234567890123456"
"#, env::temp_dir().to_str().unwrap());

    let global: Global = serde_yaml::from_str(&config).unwrap();
    let data: Vec<u8> = "Hello, world!".as_bytes().to_vec();
    let block = rt.block_on(IndirectBlock::create(&global, &data)).unwrap().to_enum();
    println!("let block = {:?}", block);
    let stored = rt.block_on(Stored::new(&global, &block)).unwrap();
    println!("let stored = {:?}", stored);
    let received = rt.block_on(async {
        let deserialized: BlockType = stored.get(&global).await.unwrap();
        println!("let deserialized = {:?}", deserialized);
        deserialized.get(&global, deserialized.range(&global).await.unwrap()).await.unwrap()
    });
    assert_eq!(data, received);
    rt.block_on(stored.delete(&global)).unwrap();
}