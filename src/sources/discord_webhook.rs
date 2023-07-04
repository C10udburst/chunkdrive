use serde::Deserialize;
use super::source::ISource;
use super::error::SourceError;
use async_trait::async_trait;

#[derive(Deserialize, Debug)]
pub struct DiscordWebhook {
    pub url: String,
}

#[derive(Deserialize)]
struct IdResponse {
    id: String
}

#[derive(Deserialize)]
struct MessageAttachment {
    url: String
}

#[derive(Deserialize)]
struct MessageResponse {
    attachments: Vec<MessageAttachment>
}

#[async_trait]
impl ISource for DiscordWebhook {
    async fn get(&self, descriptor: &[u8]) -> Result<Vec<u8>, SourceError> {
        let url = format!("{}/messages/{}", self.url, String::from_utf8(descriptor.to_vec()).unwrap());
        let client = reqwest::Client::new();
        let response = match client
            .get(&url)
            .send()
            .await {
                Ok(response) => response,
                Err(e) => return Err(SourceError::new(e.to_string()))
            };
        let file_url = match response.json::<MessageResponse>().await {
            Ok(response) => response.attachments.first().unwrap().url.clone(),
            Err(e) => return Err(SourceError::new(e.to_string()))
        };
        match client
            .get(&file_url)
            .send()
            .await {
                Ok(response) => Ok(response.bytes().await.unwrap().to_vec()),
                Err(e) => return Err(SourceError::new(e.to_string()))
            }
    }

    async fn put(&self, descriptor: &[u8], data: &[u8]) -> Result<(), SourceError> {
        let url = format!("{}/messages/{}", self.url, String::from_utf8(descriptor.to_vec()).unwrap());
        let data_part = reqwest::multipart::Part::bytes(data.to_vec())
            .file_name("data")
            .mime_str("application/octet-stream")
            .unwrap();
        let form = reqwest::multipart::Form::new().part("data", data_part);
        let client = reqwest::Client::new();
        match client
            .patch(&url)
            .multipart(form)
            .send()
            .await {
                Ok(_) => Ok(()),
                Err(e) => return Err(SourceError::new(e.to_string()))
            }
    }

    async fn delete(&self, descriptor: &[u8]) -> Result<(), SourceError> {
        let url = format!("{}/messages/{}", self.url, String::from_utf8(descriptor.to_vec()).unwrap());
        let client = reqwest::Client::new();
        match client
            .delete(&url)
            .send()
            .await {
                Ok(_) => Ok(()),
                Err(e) => return Err(SourceError::new(e.to_string()))
            }
    }

    async fn create(&self, data: &[u8]) -> Result<Vec<u8>, SourceError> {
        let data_part = reqwest::multipart::Part::bytes(data.to_vec())
            .file_name("data")
            .mime_str("application/octet-stream")
            .unwrap();
        let form = reqwest::multipart::Form::new().part("data", data_part);
        let client = reqwest::Client::new();
        let response = match client
            .post(&self.url)
            .multipart(form)
            .send()
            .await {
                Ok(response) => response,
                Err(e) => return Err(SourceError::new(e.to_string()))
            };
            
        match response.json::<IdResponse>().await {
            Ok(response) => Ok(format!("{}", response.id).into_bytes()),
            Err(e) => Err(SourceError::new(e.to_string()))
        }
    }
}   