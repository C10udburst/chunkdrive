use std::sync::Arc;
use futures::StreamExt;
use serde_yaml::from_str;

use crate::{blocks::block::{Block, BlockType}, global::Global};
use super::utils::make_temp_config;


async fn shared1(encryption: bool, local_size: usize, data: Vec<u8>) {
    let global = Arc::new(from_str::<Global>(&make_temp_config(encryption, local_size)).unwrap());
    let range = 0..data.len();
    let mut block = BlockType::create(global.clone(), data.clone(), 0).await.unwrap();
    dbg!(&block);
    assert_eq!(block.range(global.clone()).await.unwrap(), range);
    let mut got1 = Vec::new();
    {
        let mut stream = block.get(global.clone(), range.clone());
        while let Some(chunk) = stream.next().await {
            got1.extend(chunk.unwrap());
        }
        assert_eq!(got1, data);
    }
    let data1 = data.iter().map(|x| {
        let mut y = x.to_owned() as u16;
        y += 5;
        y %= 256;
        y as u8
    }).collect::<Vec<u8>>();
    let range1 = 0..data1.len();
    block.put(global.clone(), data1.clone(), range1.clone()).await.unwrap();
    let mut got2 = Vec::new();
    {
        let mut stream = block.get(global.clone(), range1.clone());
        while let Some(chunk) = stream.next().await {
            got2.extend(chunk.unwrap());
        }
        assert_eq!(got2, data1);
    }
    block.delete(global.clone()).await;
}

#[tokio::test]
async fn unencrypted_fits_in_one_block() {
    let data = vec![1u8, 2, 3, 4, 5].repeat(5);
    shared1(false, 26, data).await;
}

#[tokio::test]
async fn encrypted_fits_in_one_block() {
    let data = vec![1u8, 2, 3, 4, 5].repeat(5);
    shared1(true, 26, data).await;
}

#[tokio::test]
async fn unencrypted_fits_direct_blocks() {
    let data = vec![1u8, 2, 3, 4, 5].repeat(95);
    shared1(false, 26, data).await;
}

#[tokio::test]
async fn encrypted_fits_direct_blocks() {
    let data = vec![1u8, 2, 3, 4, 5].repeat(90);
    shared1(true, 26, data).await;
}

#[tokio::test]
async fn unencrypted_needs_indirect_blocks() {
    let data = vec![1u8, 2, 3, 4, 5].repeat(100);
    shared1(false, 16, data).await;
}