//! 加密模块

use std::path::PathBuf;
use anyhow::Result;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// 加密服务
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// 创建加密服务
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
        Self { cipher }
    }

    /// 创建带文件路径的加密服务
    pub fn from_key_file(key_path: PathBuf) -> Result<Self> {
        // 尝试加载密钥文件
        if key_path.exists() {
            let key_data = std::fs::read(&key_path)?;
            if key_data.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_data);
                return Ok(Self::new(&key));
            }
        }

        // 生成新密钥
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);

        // 保存密钥
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&key_path, &key)?;

        Ok(Self::new(&key))
    }

    /// 加密字符串
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // 组合 nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(BASE64.encode(result))
    }

    /// 解密字符串
    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let data = BASE64.decode(encrypted)?;

        if data.len() < 12 {
            anyhow::bail!("Invalid encrypted data");
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        Ok(String::from_utf8(plaintext)?)
    }
}

/// 生成随机加密密钥
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
}