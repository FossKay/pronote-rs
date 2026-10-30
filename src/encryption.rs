use aes::Aes128;
use cbc::{
    Decryptor, Encryptor,
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7},
};
use md5::{Digest, Md5};
use sha2::Sha256;

use crate::Error;

#[derive(Debug, Clone)]
pub struct Encryption {
    key: [u8; 16], // AES-128
    iv: [u8; 16],  // in CBC mode
}

impl Encryption {
    pub fn new() -> Self {
        // initial key -> md5 of empty bytes
        // initial iv -> zeroes
        Self {
            key: md5(&[]),
            iv: [0u8; 16],
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // classic AES-128-CBC encryption
        let cipher = Encryptor::<Aes128>::new(&self.key.into(), &self.iv.into());
        let mut buffer = vec![0; data.len() + 16];
        buffer[..data.len()].copy_from_slice(data);

        cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
            .map(|x| x.to_vec())
            .unwrap()
    }

    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // classic AES-128-CBC decryption
        let cipher = Decryptor::<Aes128>::new(&self.key.into(), &self.iv.into());
        let mut buffer = data.to_vec();

        cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map(|x| x.to_vec())
            .unwrap()
    }

    pub fn try_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        // classic AES-128-CBC decryption
        let cipher = Decryptor::<Aes128>::new(&self.key.into(), &self.iv.into());
        let mut buffer = data.to_vec();

        cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map(|x| x.to_vec())
            .map_err(|_| Error::InvalidCredentials)
    }

    pub fn set_iv(&mut self, iv: &[u8]) {
        self.iv = md5(iv); // every new iv is md5'd
    }

    pub fn set_key(&mut self, key: &[u8]) {
        self.key = md5(key); // every new key is md5'd
    }

    pub fn authentication_key(username: &str, password: &str, nonce: &str) -> Vec<u8> {
        // the key is md5(username + upper(hex(sha256(nonce + password))))
        // Self::set_key already handles md5'ing each new key
        let mut sha = Sha256::new();
        sha.update(nonce.as_bytes());
        sha.update(password.trim().as_bytes());

        let hash = hex::encode_upper(sha.finalize());

        format!("{username}{hash}").as_bytes().to_owned()
    }

    /* no longer used, behaviour changed in september 2026
    pub fn remove_alea(data: &str) -> String {
        data.chars().step_by(2).map(char::from).collect()
    } */
}

pub fn md5(data: &[u8]) -> [u8; 16] {
    Md5::digest(data).into()
}
