use serde::Deserialize;
use async_trait::async_trait;
use rand::{Rng, distributions};
use tokio::fs::{File, OpenOptions, remove_file};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use super::source::ISource;
use super::error::SourceError;

#[derive(Deserialize, Debug)]
pub struct Local {
    pub folder: String,
}

#[async_trait]
impl ISource for Local {
    async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError> {
        let path = format!("{}/{}", self.folder, std::str::from_utf8(descriptor).unwrap());
        let mut file = File::open(path).await.map_err(|e| SourceError::new(format!("Could not open file: {}", e)))?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.map_err(|e| SourceError::new(format!("Could not read file: {}", e)))?;
        Ok(contents)
    }

    async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError> {
        let path = format!("{}/{}", self.folder, std::str::from_utf8(descriptor).unwrap());
        let mut file = OpenOptions::new()
            .write(true)
            .create(false)
            .open(path)
            .await
            .map_err(|e| SourceError::new(format!("Could not open file: {}", e)))?;
        file.write_all(data).await.map_err(|e| SourceError::new(format!("Could not write to file: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError> {
        let path = format!("{}/{}", self.folder, std::str::from_utf8(descriptor).unwrap());
        remove_file(path).await.map_err(|e| SourceError::new(format!("Could not delete file: {}", e)))?;
        Ok(())
    }

    async fn create(&self, data: &[u8]) -> Result<Vec<u8>, SourceError> {
        let mut descriptor = rand::thread_rng().sample_iter(distributions::Alphanumeric).take(32).collect::<Vec<u8>>();
        let mut path_work = format!("{}/{}", self.folder, std::str::from_utf8(&descriptor).unwrap());
        while File::open(path_work).await.is_ok() {
            descriptor = rand::thread_rng().sample_iter(distributions::Alphanumeric).take(32).collect::<Vec<u8>>();
            path_work = format!("{}/{}", self.folder, std::str::from_utf8(&descriptor).unwrap());
        }
        let path = format!("{}/{}", self.folder, std::str::from_utf8(&descriptor).unwrap());
        let mut file = File::create(path).await.map_err(|e| SourceError::new(format!("Could not create file: {}", e)))?;
        file.write_all(data).await.map_err(|e| SourceError::new(format!("Could not write to file: {}", e)))?;
        Ok(descriptor)
    }
}

#[test]
fn test_local() {
    use std::env;
    use tokio::runtime::Runtime;
    use super::source::ISource;
    use super::local::Local;

    let rt = Runtime::new().unwrap();
    let local = Local { folder: env::temp_dir().to_str().unwrap().to_string() };
    let data1 = b"Hello World!";
    let descriptor = rt.block_on(local.create(data1)).unwrap();
    let data1_out = rt.block_on(local.get(&descriptor)).unwrap();
    assert_eq!(data1, &data1_out[..]);
    let data2 = b"Hello World! 2";
    rt.block_on(local.put(&descriptor, data2)).unwrap();
    let data2_out = rt.block_on(local.get(&descriptor)).unwrap();
    assert_eq!(data2, &data2_out[..]);
    rt.block_on(local.delete(&descriptor)).unwrap();
}