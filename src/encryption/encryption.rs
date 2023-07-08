use serde::Deserialize;

use super::{aes::Aes, none::None};

pub trait Encryption {
    fn max_size(&self, source_size: usize) -> usize;
    fn encrypt(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String>;
    fn decrypt(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum EncryptionType {
    #[serde(rename = "none")]
    None(None),
    #[serde(rename = "aes")]
    Aes(Aes)
}

impl Default for EncryptionType {
    fn default() -> Self { EncryptionType::None(None) }
}

// This macro removes the need to write out the match statement for each method in the enum
macro_rules! match_method {
    ($self:ident, $method:ident, $($arg:expr),*) => {
        match $self {
            EncryptionType::None(encryption) => encryption.$method($($arg),*),
            EncryptionType::Aes(encryption) => encryption.$method($($arg),*)
        }
    };
}

impl EncryptionType {
    pub fn human_readable(&self) -> &str {
        match self {
            EncryptionType::None(_) => "none",
            EncryptionType::Aes(_) => "aes"
        }
    }
}

impl Encryption for EncryptionType {
    fn max_size(&self, source_size: usize) -> usize {
        match_method!(self, max_size, source_size)
    }

    fn encrypt<'a>(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String> {
        match_method!(self, encrypt, data, iv)
    }

    fn decrypt<'a>(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String> {
        match_method!(self, decrypt, data, iv)
    }
}