use serde_yaml::from_str;
use tokio::runtime::Runtime;

use crate::global::Global;
use super::utils::make_temp_config;

fn shared_default(encryption: bool) {
    let cfg = make_temp_config(encryption, 25);
    let global = from_str::<Global>(&cfg).unwrap();

    let data = vec![1u8, 2, 3, 4, 5].repeat(5);
    let bucket = global.get_bucket(global.random_bucket().unwrap().as_str()).unwrap();
    
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let data_clone = data.clone();
        let descriptor = bucket.create().await.unwrap();
        bucket.put(&descriptor, data_clone).await.unwrap();
        let data2 = bucket.get(&descriptor).await.unwrap();
        assert_eq!(data, data2);
        bucket.delete(&descriptor).await.unwrap();
        match bucket.get(&descriptor).await {
            Ok(_) => panic!("Descriptor should not exist"),
            Err(_) => (),
        }
    });
}

#[test]
fn simple_unencrypted() {
    shared_default(false);
}

#[test]
fn simple_encrypted() {
    shared_default(true);
}