use std::sync::Arc;
use serde_yaml::from_str;

use crate::{blocks::{block::Block, direct_block::DirectBlock}, global::Global};
use super::utils::make_temp_config;

#[tokio::test]
async fn empty_data() {
    let global = Arc::new(from_str::<Global>(&make_temp_config(false, 30)).unwrap());
    let data = Vec::new();
    let block = DirectBlock::create(global.clone(), data.clone(), 0).await;
    assert!(block.is_err());
}