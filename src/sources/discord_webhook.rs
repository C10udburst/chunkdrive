use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::source::Source;

#[derive(Debug, Deserialize)]
pub struct DiscordWebhook {
    url: String,
}

/* #region discord schema */
#[derive(Deserialize)]
struct MessageResponse {
    id: String,
    attachments: Vec<MessageAttachment>,
}

#[derive(Deserialize)]
struct MessageAttachment {
    url: String,
}

/* #endregion */

#[async_trait]
impl Source for DiscordWebhook {
    fn max_size(&self) -> usize {
        1024 * 1024 * 24
    }

    async fn get(&self, descriptor: &String) -> Result<Vec<u8>, String> {
        let url = format!("{}/messages/{}", self.url, descriptor);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        let parsed = response
            .json::<MessageResponse>()
            .await
            .map_err(|e| format!("Error parsing response: {}", e))?;
        if parsed.attachments.is_empty() {
            return Err("No attachments found".to_string());
        }
        match client.get(&parsed.attachments[0].url).send().await {
            Ok(response) => Ok(response.bytes().await
                .map_err(|e| format!("Error reading response: {}", e))?.to_vec()),
            Err(e) => Err(format!("Error sending request: {}", e))
        }
    }

    async fn put(&self, descriptor: &String, data: Vec<u8>) -> Result<(), String> {
        let url = format!("{}/messages/{}", self.url, descriptor);
        let client = reqwest::Client::new();
        let data_part = reqwest::multipart::Part::bytes(data)
            .file_name("d")
            .mime_str("application/octet-stream")
            .map_err(|e| format!("Error creating part: {}", e))?;
        let payload_part = reqwest::multipart::Part::text(json!({
           "attachments": [
               {
                   "id": 0,
                   "filename": "d"
               }
           ],
        }).to_string())
            .mime_str("application/json")
            .map_err(|e| format!("Error creating part: {}", e))?;
        let form = reqwest::multipart::Form::new()
            .part("payload_json", payload_part)
            .part("files[0]", data_part);
        client
            .patch(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        Ok(())
    }

    async fn delete(&self, descriptor: &String) -> Result<(), String> {
        let url = format!("{}/messages/{}", self.url, descriptor);
        let client = reqwest::Client::new();
        client
            .delete(&url)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        Ok(())
    }

    async fn create(&self) -> Result<String, String> {
        let client = reqwest::Client::new();
        let empty = reqwest::multipart::Part::bytes(Vec::new())
            .file_name("d")
            .mime_str("application/octet-stream")
            .map_err(|e| format!("Error creating part: {}", e))?;
        let payload_part = reqwest::multipart::Part::text(json!({
            "flags": 1<<12, // suppress notifications (@silent) and embeds
            "attachments": [
                {
                    "id": 0,
                    "filename": "d"
                }
            ],
         }).to_string())
             .mime_str("application/json")
             .map_err(|e| format!("Error creating part: {}", e))?;
        let form = reqwest::multipart::Form::new()
            .part("payload_json", payload_part)
            .part("files[0]", empty);
        let response = client
            .post(&self.url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        let parsed = response
            .json::<MessageResponse>()
            .await
            .map_err(|e| format!("Error parsing response: {}", e))?;
        Ok(parsed.id)
    }
}
