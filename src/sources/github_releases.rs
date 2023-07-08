use async_trait::async_trait;
use rand::{thread_rng, distributions::Alphanumeric, Rng};
use reqwest::header::{HeaderMap, ACCEPT, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use serde_json::json;

use crate::global::Descriptor;
use super::source::Source;

#[derive(Debug, Deserialize)]
pub struct GithubReleases {
    owner: String,
    repo: String,
    pat: String,

    #[serde(default = "default_descriptor_length")]
    descriptor_length: usize,
}

const fn default_descriptor_length() -> usize { 16 }

#[derive(Deserialize)]
pub struct ReleaseResponse {
    id: u64,
    #[serde(default)]
    assets: Vec<AssetPart>,
}

#[derive(Deserialize)]
pub struct AssetPart {
    id: u64,
}

impl GithubReleases {
    fn make_headers(&self, mime: Option<&str>, accept: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(accept) = accept {
            headers.insert(ACCEPT, HeaderValue::from_str(accept).unwrap());
        } else {
            headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
        }
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.pat)).unwrap());
        headers.insert(USER_AGENT, HeaderValue::from_static("reqwest/0.11.3"));
        if let Some(mime) = mime {
            headers.insert("Content-Type", HeaderValue::from_str(mime).unwrap());
        }
        headers
    }
}


#[async_trait]
impl Source for GithubReleases {
    fn max_size(&self) -> usize {
        1024 * 1024 * 1024 // 1 GB
    }

    async fn get(&self, descriptor: &Descriptor) -> Result<Vec<u8>, String> {
        let tag = std::str::from_utf8(&descriptor)
            .map_err(|e| format!("Error parsing descriptor: {}", e))?;
        
        // Get release info
        let url = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", self.owner, self.repo, tag);
        let client = reqwest::Client::new();
        let parsed = client
            .get(&url)
            .headers(self.make_headers(None, None))
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?
            .json::<ReleaseResponse>()
            .await
            .map_err(|e| format!("Error parsing response: {}", e))?;

        // Get asset id
        let id = parsed.assets.first()
            .ok_or_else(|| format!("No assets found for release {}", tag))?
            .id;

        let url = format!("https://api.github.com/repos/{}/{}/releases/assets/{}", self.owner, self.repo, id);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .headers(self.make_headers(None, Some("application/octet-stream")))
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        Ok(response.bytes().await
            .map_err(|e| format!("Error reading response: {}", e))?.to_vec())
    }

    async fn put(&self, descriptor: &Descriptor, data: Vec<u8>) -> Result<(), String> {
        let tag = std::str::from_utf8(&descriptor)
            .map_err(|e| format!("Error parsing descriptor: {}", e))?;
        
        // Get release info
        let url = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", self.owner, self.repo, tag);
        let client = reqwest::Client::new();
        let parsed = client
            .get(&url)
            .headers(self.make_headers(None, None))
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?
            .json::<ReleaseResponse>()
            .await
            .map_err(|e| format!("Error parsing response: {}", e))?;
        
        // Delete existing asset
        for asset in parsed.assets {
            let url = format!("https://api.github.com/repos/{}/{}/releases/assets/{}", self.owner, self.repo, asset.id);
            client
                .delete(&url)
                .headers(self.make_headers(None, None))
                .send()
                .await
                .map_err(|e| format!("Error sending request: {}", e))?;
        }
        
        // Upload new asset
        let url = format!("https://uploads.github.com/repos/{}/{}/releases/{}/assets?name=d.bin", self.owner, self.repo, parsed.id);
        let response = client
            .post(&url)
            .headers(self.make_headers(Some("application/octet-stream"), None))
            .body(data)
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        if !response.status().is_success() {
            return Err(format!("Error uploading asset: {}", response.text().await
                .map_err(|e| format!("Error reading response: {}", e))?));
        }
        Ok(())
    }

    async fn delete(&self, descriptor: &Descriptor) -> Result<(), String> {
        let tag = std::str::from_utf8(&descriptor)
            .map_err(|e| format!("Error parsing descriptor: {}", e))?;
        
        let mut errors = Vec::new();

        // Get release info
        let url = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", self.owner, self.repo, tag);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .headers(self.make_headers(None, None))
            .send()
            .await;
        if response.is_err() {
            errors.push(format!("Error sending request: {}", response.err().unwrap()));
        } else {
            let parsed = response
                .unwrap()
                .json::<ReleaseResponse>()
                .await
                .map_err(|e| format!("Error parsing response: {}", e));

            if parsed.is_err() {
                errors.push(parsed.err().unwrap());
            } else {
                // Delete existing asset(s)
                let id = parsed.as_ref().unwrap().id.clone();
                for asset in parsed.unwrap().assets {
                    let url = format!("https://api.github.com/repos/{}/{}/releases/assets/{}", self.owner, self.repo, asset.id);
                    match client
                        .delete(&url)
                        .headers(self.make_headers(None, None))
                        .send()
                        .await {
                            Ok(_) => (),
                            Err(e) => errors.push(format!("Error deleting asset: {}", e))
                        }
                }

                // Delete release
                let url = format!("https://api.github.com/repos/{}/{}/releases/{}", self.owner, self.repo, id);
                match client
                    .delete(&url)
                    .headers(self.make_headers(None, None))
                    .send()
                    .await {
                        Ok(_) => (),
                        Err(e) => errors.push(format!("Error deleting release: {}", e))
                    }
            }
        }

        // Delete tag
        let url = format!("https://api.github.com/repos/{}/{}/git/refs/tags/{}", self.owner, self.repo, tag);
        client
            .delete(&url)
            .headers(self.make_headers(None, None))
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
    
        if !errors.is_empty() {
            return Err(errors.join(", "));
        }
        Ok(())
    }

    async fn create(&self) -> Result<Descriptor, String> {
        let mut descriptor = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.descriptor_length)
            .map(char::from)
            .collect::<String>();

        let client = reqwest::Client::new();

        // Check if the descriptor already exists
        let mut url = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", self.owner, self.repo, descriptor);
        loop {
            let response = client
                .get(&url)
                .headers(self.make_headers(None, None))
                .send()
                .await.map_err(|e| format!("Error sending request: {}", e))?;
            if response.status() == 404 {
                break;
            } else if !response.status().is_success() {
                return Err(format!("Error checking if release exists: {}", response.text().await
                    .map_err(|e| format!("Error reading response: {}", e))?));
            } else {
                descriptor = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(self.descriptor_length)
                    .map(char::from)
                    .collect::<String>();
                url = format!("https://api.github.com/repos/{}/{}/releases/tags/{}", self.owner, self.repo, descriptor);
            }
        }

        // Create release
        let url = format!("https://api.github.com/repos/{}/{}/releases", self.owner, self.repo);
        let response = client
            .post(&url)
            .headers(self.make_headers(Some("application/json"), None))
            .body(json!({
                "tag_name": descriptor,
                "name": descriptor,
                "body": "",
                "draft": false,
                "prerelease": true
            }).to_string())
            .send()
            .await
            .map_err(|e| format!("Error sending request: {}", e))?;
        if !response.status().is_success() {
            return Err(format!("Error creating release: {}", response.text().await
                .map_err(|e| format!("Error reading response: {}", e))?));
        }

        Ok(descriptor.into_bytes())
    }
}