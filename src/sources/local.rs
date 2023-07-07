use async_trait::async_trait;
use serde::Deserialize;
use tokio::{io::{BufReader, AsyncReadExt, AsyncWriteExt}, fs::{File, remove_file, OpenOptions}};
use rand::{thread_rng, Rng, distributions::Alphanumeric};

use super::source::Source;


#[derive(Debug, Deserialize)]
pub struct LocalSource {
    folder: String,
    #[serde(default = "default_max_size")]
    max_size: usize,
}

const fn default_max_size() -> usize {
    1024 * 1024 * 1024
}

#[async_trait]
impl Source for LocalSource {
    fn max_size(&self) -> usize {
        self.max_size
    }

    async fn get(&self, descriptor: String) -> Result<Vec<u8>, String> {
        let file_path = format!("{}/{}", self.folder, descriptor);
        let file = match File::open(file_path).await {
            Ok(file) => file,
            Err(_) => return Err("File not found".to_string())
        };
        let mut data = Vec::new();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut data).await.map_err(|e| format!("Error reading file: {}", e))?;
        Ok(data)
    }

    async fn put(&self, descriptor: String, data: Vec<u8>) -> Result<(), String> {
        let file_path = format!("{}/{}", self.folder, descriptor);
        let mut file = match OpenOptions::new()
            .write(true)
            .create(false)     // We don't want to create the file if it doesn't exist, it should already exist,
            .truncate(true)    // as we only should create files with ::create() to ensure safe descriptors
            .open(file_path)
        .await {
            Ok(file) => file,
            Err(e) => return Err(format!("Error opening file: {}", e))
        };
        // Write the data to the file
        file.write(&data.to_vec()).await.map_err(|e| format!("Error writing file: {}", e))?;
        Ok(())
    }

    async fn delete(&self, descriptor: String) -> Result<(), String> {
        let file_path = format!("{}/{}", self.folder, descriptor);
        match remove_file(file_path).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error deleting file: {}", e))
        }
    }

    async fn create(&self) -> Result<String, String> {
        let mut descriptor = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect::<String>();
        let mut file_path = format!("{}/{}", self.folder, descriptor);
        // Ensure that the descriptor is unique
        while File::open(file_path).await.is_ok() {
            descriptor = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(24)
                .map(char::from)
                .collect::<String>();
            file_path = format!("{}/{}", self.folder, descriptor);
        }
        file_path = format!("{}/{}", self.folder, descriptor); // this is necessary because the file_path variable is moved into the closure below
        let mut file = File::create(file_path).await.map_err(|e| format!("Error creating file: {}", e))?;
        file.write_all(b"").await.map_err(|e| format!("Error writing file: {}", e))?;
        Ok(descriptor)
    }
}