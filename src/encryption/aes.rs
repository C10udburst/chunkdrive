use super::encryption::IEncryption;
use serde::Deserialize;
use async_trait::async_trait;
use crypto::{aes, blockmodes, buffer::{WriteBuffer, ReadBuffer}};

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