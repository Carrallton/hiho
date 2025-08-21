use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use argon2::{Argon2, Params, PasswordHasher, password_hash::SaltString};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: [u8; 16],
}

pub fn derive_key(password: &str, salt: &str) -> Result<[u8; 32], Box<dyn Error>> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    );
    let salt = SaltString::encode_b64(salt.as_bytes())
        .map_err(|e| format!("Salt encoding error: {}", e))?;
    
    let hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing error: {}", e))?;
    
    let key: [u8; 32] = hash.hash.ok_or("Hash is None")?
        .as_bytes()[..32].try_into()
        .map_err(|_| "Key conversion error")?;
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<EncryptedData, Box<dyn Error>> {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    
    let cipher = Aes256CbcEnc::new_from_slices(key, &iv)
        .map_err(|e| format!("Cipher creation error: {}", e))?;
    
    // Создаем буфер с запасом для padding
    let mut buffer = vec![0u8; data.len() + 16];
    buffer[..data.len()].copy_from_slice(data);
    
    let ciphertext = cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
        .map_err(|e| format!("Encryption error: {}", e))?
        .to_vec();
    
    Ok(EncryptedData { ciphertext, iv })
}

pub fn decrypt(encrypted: &EncryptedData, key: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256CbcDec::new_from_slices(key, &encrypted.iv)
        .map_err(|e| format!("Cipher creation error: {}", e))?;
    
    // Создаем буфер для дешифрования
    let mut buffer = encrypted.ciphertext.clone();
    
    let plaintext = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("Decryption error (possibly wrong password): {}", e))?
        .to_vec();
    
    Ok(plaintext)
}