use magic_crypt::{new_magic_crypt, MagicCryptTrait, MagicCrypt128, MagicCryptError};
use serde::{Deserialize, Serialize};

pub struct Encryptor {
    mc: MagicCrypt128,
}

impl Encryptor {
    pub fn new(key: String) -> Self {
        Self {
            mc: new_magic_crypt!(key.as_str(), 128),
        }
    }

    pub fn from_bytes(key: [u8; 32]) -> Self {
        Self {
            mc: new_magic_crypt!(String::from_utf8_lossy(&key), 128),
        }
    }
    
    /*
    pub fn new_bits(key: [u8; 32]) -> Self {
        Self {
            mc: new_magic_crypt!(key, 256),
        }
    }
     */

    pub fn encrypt(&self, input: String) -> String {
        self.mc.encrypt_str_to_base64(input.as_str())
    }

    pub fn decrypt(&self, input: String) -> Result<String, MagicCryptError> {
        self.mc.decrypt_base64_to_string(input)
    }

    pub fn encrypt_bytes(&self, bytes: Vec<u8>) -> String {
        self.mc.encrypt_bytes_to_base64(&bytes)
    }

    pub fn decrypt_bytes(&self, bytes: Vec<u8>) -> Result<Vec<u8>, MagicCryptError> {
        self.mc.decrypt_bytes_to_bytes(&bytes)
    }
}


