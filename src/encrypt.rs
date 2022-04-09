use magic_crypt::{new_magic_crypt, MagicCryptTrait, MagicCrypt128, MagicCryptError};

pub struct Encryptor {
    mc: MagicCrypt128,
}

impl Encryptor {
    pub fn new(key: String) -> Self {
        Self {
            mc: new_magic_crypt!(key.as_str(), 128),
        }
    }

    pub fn encrypt(&self, input: String) -> String {
        self.mc.encrypt_str_to_base64(input.as_str())
    }

    pub fn decrypt(&self, input: String) -> Result<String, MagicCryptError> {
        self.mc.decrypt_base64_to_string(input)
    }
}