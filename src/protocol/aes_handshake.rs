use std::io;

use openssl::{
    encrypt::{Decrypter, Encrypter},
    pkey::{PKey, Private, Public},
    rsa::Padding,
};
use serde::{Deserialize, Serialize};

use super::AES_KEY_SIZE;

const PADDING: Padding = Padding::PKCS1_OAEP;

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
        let mut decrypted_buff = vec![0; decryptor.decrypt_len(&self.encrypted_aes_key)?];
        decryptor.decrypt(&self.encrypted_aes_key, &mut decrypted_buff)?;

        let mut key = [0; AES_KEY_SIZE];
        key.copy_from_slice(&decrypted_buff[..AES_KEY_SIZE]);
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::{rand::rand_bytes, rsa::Rsa};

    #[test]
    fn test_decrypt_key() {
        let mut aes_key = [0; AES_KEY_SIZE];
        rand_bytes(&mut aes_key).unwrap();

        let private_key = PKey::from_rsa(Rsa::generate(1024).unwrap()).unwrap();
        let public_raw = private_key.public_key_to_pem().unwrap();
        let public_key = PKey::public_key_from_pem(&public_raw).unwrap();

        let handshake = AesHandshake::new(&aes_key, &public_key).unwrap();
        let decrypted_key = handshake.decrypt_key(&private_key).unwrap();
        assert_eq!(decrypted_key, aes_key);
    }
}
