use serde_yaml::from_reader;
use std::fs::File;

mod global;
mod source;
mod sources;
mod encryption;
mod block;
mod stored;
mod inodes;
mod cli;

use global::Global;


#[tokio::main(flavor = "current_thread")]
async fn main() {
    let file = File::open("config.yml").unwrap();
    let global: Global = from_reader(file).unwrap();
    let shell: bool = std::env::args().any(|arg| arg == "--shell");
    if shell {
      cli::shell(global).await;
    }
  }


#[test]
fn stored_single_block() {
    use std::env;
    use tokio::runtime::Runtime;
    use block::{indirect_block::IndirectBlock, block::BlockType};
    use crate::{stored::Stored};

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
services:
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
        deserialized.get(&global, &deserialized.range(&global).await.unwrap()).await.unwrap()
    });
    assert_eq!(data, received);
    rt.block_on(stored.delete(&global)).unwrap();
}