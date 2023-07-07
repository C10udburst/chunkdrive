use serde::Deserialize;

use super::encryption::Encryption;

#[derive(Deserialize, Debug)]
pub struct None;

impl Encryption for None {
    fn encrypt(&self, data: Vec<u8>, _iv: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(data)
    }

    fn decrypt<'a>(&self, data: Vec<u8>, _iv: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(data)
    }
}