use std::io;

use openssl::{
    encrypt::{Decrypter, Encrypter},
    pkey::{PKey, Private, Public},
    rsa::Padding,
};
use serde::{Deserialize, Serialize};

use super::AES_KEY_SIZE;

const PADDING: Padding = Padding::PKCS1;

#[derive(Serialize, Deserialize)]
pub struct AesHandshake {
    encrypted_aes_key: Vec<u8>,
}

impl AesHandshake {
    pub fn new(aes_key: &[u8; AES_KEY_SIZE], public_key: &PKey<Public>) -> Result<Self, io::Error> {
        let mut encryptor = Encrypter::new(public_key)?;
        encryptor.set_rsa_padding(PADDING)?;
        let mut encrypted_aes_key = vec![0; encryptor.encrypt_len(aes_key)?];
        encryptor.encrypt(aes_key, &mut encrypted_aes_key)?;

        Ok(AesHandshake { encrypted_aes_key })
    }

    pub fn decrypt_key(
        &self,
        private_key: &PKey<Private>,
    ) -> Result<[u8; AES_KEY_SIZE], io::Error> {
        let mut decryptor = Decrypter::new(private_key)?;
        decryptor.set_rsa_padding(PADDING)?;
        let mut decrypted_data = [0; AES_KEY_SIZE];
        decryptor.decrypt(&self.encrypted_aes_key, &mut decrypted_data)?;
        Ok(decrypted_data)
    }
}
