use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::stream::BoxStream;
use serde::Deserialize;
use tokio::{io::{BufReader, AsyncReadExt, AsyncWriteExt}, fs::{File, remove_file, OpenOptions}};

use super::source::Source;


#[derive(Debug, Deserialize)]
pub struct LocalSource {
    folder: String,
}

#[async_trait]
impl Source for LocalSource {
    fn get(&self, descriptor: String) -> BoxStream<Result<Bytes, String>> {
        let file_path = format!("{}/{}", self.folder, descriptor);
        Box::pin(async_stream::stream! {
            let file = match File::open(file_path).await {
                Ok(file) => file,
                Err(e) => {
                    yield Err(format!("Error opening file: {}", e));
                    return;
                }
            };
            let mut reader = BufReader::new(file);
            let mut buf = BytesMut::new();
            loop {
                match reader.read_buf(&mut buf).await {
                    Ok(0) => break,
                    Ok(_) => yield Ok(buf.clone().freeze()), // TODO: Is this clone necessary?
                    Err(e) => {
                        yield Err(format!("Error reading file: {}", e));
                        return;
                    }
                }
            }
        })
    }

    async fn put(&self, descriptor: String, data: Bytes) -> Result<(), String> {
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
}