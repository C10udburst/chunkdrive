use std::sync::Arc;
use serde_yaml::from_str;

use crate::{global::Global, stored::Stored};
use super::utils::make_temp_config;

#[tokio::test]
async fn stored_with_url() {
    let global = Arc::new(from_str::<Global>(&make_temp_config(false, 30)).unwrap());
    let object = "Hello".to_string();
    let stored = Stored::create(global.clone(), object.clone()).await.unwrap();
    let url = stored.as_url();
    let split = url.split('$').collect::<Vec<&str>>();
    assert!(split.len() == 2);
    let (bucket, descriptor) = (split[0], split[1]);
    let stored1 = Stored::from_url(bucket, descriptor).unwrap();
    assert_eq!(stored, stored1);
    let object1 = stored1.get::<String>(global.clone()).await.unwrap();
    assert_eq!(object, object1);
}