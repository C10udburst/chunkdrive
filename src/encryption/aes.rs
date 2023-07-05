use super::encryption::IEncryption;
use serde::Deserialize;
use async_trait::async_trait;
use crypto::{aes, blockmodes, buffer::{WriteBuffer, ReadBuffer}};
use tokio::runtime::Runtime;

#[derive(Deserialize, Debug)]
pub struct Aes {
    pub key: String,
    pub iv: String,
}



#[async_trait]
impl IEncryption for Aes {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, ()> {
        let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            &self.key.as_bytes(),
            &self.iv.as_bytes(),
            blockmodes::PkcsPadding,
        );
        let mut result = Vec::<u8>::new();
        let mut out = [0; 4096];
        let mut reader = crypto::buffer::RefReadBuffer::new(data);
        let mut writer = crypto::buffer::RefWriteBuffer::new(&mut out);

        loop {
            let read = encryptor
                .encrypt(&mut reader, &mut writer, true)
                .unwrap();
            result.extend(writer.take_read_buffer().take_remaining().iter().map(|&i| i));
            match read {
                crypto::buffer::BufferResult::BufferUnderflow => break,
                crypto::buffer::BufferResult::BufferOverflow => {}
            }
        }

        Ok(result)
    }

    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ()> {
        let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            &self.key.as_bytes(),
            &self.iv.as_bytes(),
            blockmodes::PkcsPadding,
        );
        let mut result = Vec::<u8>::new();
        let mut out = [0; 4096];
        let mut reader = crypto::buffer::RefReadBuffer::new(data);
        let mut writer = crypto::buffer::RefWriteBuffer::new(&mut out);

        loop {
            let read = decryptor
                .decrypt(&mut reader, &mut writer, true)
                .unwrap();
            result.extend(writer.take_read_buffer().take_remaining().iter().map(|&i| i));
            match read {
                crypto::buffer::BufferResult::BufferUnderflow => break,
                crypto::buffer::BufferResult::BufferOverflow => {}
            }
        }
        
        Ok(result)
    }
}

#[test]
fn test_aes() {
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    let aes = Aes {
        key: "01234567890123456789012345678901".to_string(),
        iv: "0123456789012345".to_string(),
    };
    let data = b"Esse consectetur quod ut corporis repellat. Quis voluptatem natus et nostrum nulla qui quo cum.";
    let encrypted = rt.block_on(aes.encrypt(data)).unwrap();
    assert_ne!(data, &encrypted[..]);
    let decrypted = rt.block_on(aes.decrypt(&encrypted)).unwrap();
    assert_eq!(data, &decrypted[..]);
}