use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use thiserror::Error;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid data")]
    InvalidData,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Key derivation failed")]
    KeyDerivationFailed,
}

pub struct SecureStore {
    key: [u8; 32],
}

impl SecureStore {
    pub fn from_password(password: &str, salt: &[u8; SALT_LEN]) -> Result<Self, CryptoError> {
        let params =
            Params::new(65536, 3, 4, Some(32)).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;

        Ok(Self { key })
    }

    pub fn generate_salt() -> [u8; SALT_LEN] {
        let mut salt = [0u8; SALT_LEN];
        rand::rng().fill_bytes(&mut salt);
        salt
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|_| CryptoError::EncryptionFailed)?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Format: nonce || ciphertext
        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < NONCE_LEN {
            return Err(CryptoError::InvalidData);
        }

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|_| CryptoError::DecryptionFailed)?;
        let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
        let ciphertext = &data[NONCE_LEN..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        Ok(plaintext)
    }

    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let salt = SecureStore::generate_salt();
        let store = SecureStore::from_password("test_password", &salt).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = store.encrypt(plaintext).unwrap();
        let decrypted = store.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let salt = SecureStore::generate_salt();
        let store1 = SecureStore::from_password("password1", &salt).unwrap();
        let store2 = SecureStore::from_password("password2", &salt).unwrap();

        let plaintext = b"Secret data";
        let encrypted = store1.encrypt(plaintext).unwrap();

        // Decryption with wrong password should fail
        assert!(store2.decrypt(&encrypted).is_err());
    }
}
