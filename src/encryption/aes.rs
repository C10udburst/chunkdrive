use crypto::{
    aes::{self, KeySize},
    blockmodes,
    buffer::{self, ReadBuffer, WriteBuffer},
};
use serde::Deserialize;

use super::encryption::Encryption;

#[derive(Deserialize, Debug)]
pub struct Aes {
    key: String,
    #[serde(default)]
    #[serde(rename = "variant")]
    size: AesType,
}

/* #region AesType */
#[derive(Deserialize, Debug)]
pub enum AesType {
    #[serde(rename = "aes128")]
    Aes128,
    #[serde(rename = "aes192")]
    Aes192,
    #[serde(rename = "aes256")]
    Aes256,
}

impl Default for AesType {
    fn default() -> Self {
        AesType::Aes128
    }
}

impl AesType {
    fn to_enum(&self) -> KeySize {
        match self {
            AesType::Aes128 => KeySize::KeySize128,
            AesType::Aes192 => KeySize::KeySize192,
            AesType::Aes256 => KeySize::KeySize256,
        }
    }

    fn key_size(&self) -> usize {
        match self {
            AesType::Aes128 => 16,
            AesType::Aes192 => 24,
            AesType::Aes256 => 32,
        }
    }
    
    fn iv_size(&self) -> usize {
        match self {
            AesType::Aes128 => 16,
            AesType::Aes192 => 24,
            AesType::Aes256 => 32,
        }
    }
}
/* #endregion */

// generate a key from a string by repeating it (if key was shorter, we also do some bit shifting in the repetions to make it more random)
fn to_size(init_key: &Vec<u8>, size: usize) -> Vec<u8> {
    let mut key = init_key.iter().cycle().take(size).cloned().collect::<Vec<u8>>();
    for i in init_key.len()..key.len() {
        let mut tmp = key[i] as u16;
        tmp = tmp << (i%3) | tmp >> (8 - (i%3));
        tmp = tmp + key[i- init_key.len()] as u16;
        tmp = tmp % 256;
        key[i] = tmp as u8;
    }
    key
}

impl Encryption for Aes {
    fn encrypt(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut encryptor = aes::cbc_encryptor(
            self.size.to_enum(),
            &to_size(&self.key.as_bytes().to_vec(), self.size.key_size()),
            &to_size(&iv, self.size.iv_size()),
            blockmodes::PkcsPadding,
        );

        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(&data.as_slice());
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = encryptor
                .encrypt(&mut read_buffer, &mut write_buffer, true)
                .map_err(|_| format!("Symmetric encryption failed"))?;
            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );
            match result {
                buffer::BufferResult::BufferUnderflow => break,
                buffer::BufferResult::BufferOverflow => {}
            }
        }

        Ok(final_result)
    }

    fn decrypt(&self, data: Vec<u8>, iv: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut decryptor = aes::cbc_decryptor(
            self.size.to_enum(),
            &to_size(&self.key.as_bytes().to_vec(), self.size.key_size()),
            &to_size(&iv, self.size.iv_size()),
            blockmodes::PkcsPadding,
        );

        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(&data.as_slice());
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = decryptor
                .decrypt(&mut read_buffer, &mut write_buffer, true)
                .map_err(|_| format!("Symmetric decryption failed"))?;
            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );
            match result {
                buffer::BufferResult::BufferUnderflow => break,
                buffer::BufferResult::BufferOverflow => {}
            }
        }

        Ok(final_result)
    }
}

#[cfg(test)]
mod aes_tests {
    use crate::encryption;

    use super::*;

    #[test]
    fn simple() {
        let aes = Aes {
            key: "c3VwZXJzZWNyZXQ=".to_string(),
            size: AesType::Aes128,
        };
        let data = "Perferendis nihil quidem neque sed blanditiis."
            .as_bytes()
            .to_vec();
        let iv = "0123f9abcdef".as_bytes().to_vec();
        let encrypted = aes.encrypt(data.clone(), iv.clone()).unwrap();
        let encrypted_copy = encrypted.clone();
        assert_ne!(encrypted, data);
        let decrypted = aes.decrypt(encrypted, iv).unwrap();
        assert_eq!(decrypted, data);

        let iv2 = "0123f9abcde".as_bytes().to_vec();
        let encrypted2 = aes.encrypt(data.clone(), iv2.clone()).unwrap();

        assert_ne!(encrypted2, encrypted_copy);
    }

    #[test]
    fn key_extender() {
        let input_key = '0';
        let key = vec![input_key as u8];
        
        let extended = to_size(&key, 126);
        println!("{:?}", extended);

        // calculate how often the single byte of the initial key is repeated
        let input_count = extended.iter().filter(|&x| *x == input_key as u8).count();
        // make sure its less than 50%
        assert!(input_count < 125/2);

        let extended2 = to_size(&key, 126);
        // check if they are the same
        assert_eq!(extended, extended2);
    }
}