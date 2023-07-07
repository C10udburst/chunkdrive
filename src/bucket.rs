/*
    Bucket is an abstraction over a source, it includes additional features like encryption or caching.
    Each bucket has a maximum size, which is the maximum size of a single data chunk that can be stored in the bucket.
*/

use serde::Deserialize;

use crate::{sources::source::{SourceType, Source}, encryption::encryption::{EncryptionType, Encryption}};

#[derive(Deserialize, Debug)]
pub struct Bucket {
    source: SourceType,
    #[serde(default)]
    encryption: EncryptionType,
}

impl Bucket {
    // Returns the maximum size of data that can be stored in a single descriptor
    pub fn max_size(&self) -> usize {
        self.encryption.max_size(
            self.source.max_size()
        )
    }

    // Takes a descriptor and returns a stream of data or an error (String)
    pub async fn get(&self, descriptor: &String) -> Result<Vec<u8>, String> {
        let iv = descriptor.as_bytes().to_vec();
        let data = self.source.get(descriptor).await?;
        let decrypted = self.encryption.decrypt(data, iv)?;
        Ok(decrypted)
    }
    
    // Takes a descriptor and data and uploads the data to the descriptor or returns an error (String)
    pub async fn put(&self, descriptor: &String, data: Vec<u8>) -> Result<(), String> {
        let iv = descriptor.as_bytes().to_vec();
        let encrypted = self.encryption.encrypt(data, iv)?;
        self.source.put(descriptor, encrypted).await
    }
    
    // Takes a descriptor and deletes the data at the descriptor or returns an error (String)
    pub async fn delete(&self, descriptor: &String) -> Result<(), String> {
        self.source.delete(descriptor).await
    }

    // Creates a new descriptor and returns it or returns an error (String)
    pub async fn create(&self) -> Result<String, String> {
        self.source.create().await
    }
}


#[cfg(test)]
mod bucket_tests {
    use serde_yaml::from_str;
    use tokio::runtime::Runtime;

    use crate::{global::Global, tests::make_temp_config};


    #[test]
    fn bucket_test1() {
        let cfg = make_temp_config(false, 25);
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
    fn bucket_test2() {
        let cfg = make_temp_config(true, 25);
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
}