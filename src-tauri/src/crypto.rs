//! 安全的加密模块，用于API密钥等敏感数据的加密存储

use ring::aead::{self, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose};
use std::sync::Mutex;

/// 加密错误类型
#[derive(Debug)]
pub enum CryptoError {
    RandomError,
    EncryptionError,
    DecryptionError,
    InvalidData,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::RandomError => write!(f, "随机数生成失败"),
            CryptoError::EncryptionError => write!(f, "加密失败"),
            CryptoError::DecryptionError => write!(f, "解密失败"),
            CryptoError::InvalidData => write!(f, "无效的数据格式"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// 安全的密钥管理
pub struct CryptoManager {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl CryptoManager {
    /// 创建新的加密管理器
    pub fn new() -> Result<Self, CryptoError> {
        let mut key_bytes = [0u8; 32];
        let rng = SystemRandom::new();
        
        // 生成随机密钥
        rng.fill(&mut key_bytes).map_err(|_| CryptoError::RandomError)?;
        
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| CryptoError::EncryptionError)?;
        let key = LessSafeKey::new(unbound_key);
        
        Ok(Self { key, rng })
    }
    
    /// 加密字符串（主要用于API密钥）
    pub fn encrypt_string(&self, text: &str) -> Result<String, CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| CryptoError::RandomError)?;
        
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        // 准备数据：数据 + 16字节的tag空间
        let mut data = text.as_bytes().to_vec();
        let original_len = data.len();
        data.extend_from_slice(&[0u8; 16]); // 为tag预留空间
        
        // 加密数据
        let tag = self.key.seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut data)
            .map_err(|_| CryptoError::EncryptionError)?;
        
        // 移除tag预留空间，添加实际的tag
        data.truncate(original_len);
        data.extend_from_slice(tag.as_ref());
        
        // 组合格式: nonce(12) + ciphertext + tag(16)
        let mut result = Vec::with_capacity(12 + data.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&data);
        
        Ok(general_purpose::STANDARD.encode(result))
    }
    
    /// 解密字符串
    pub fn decrypt_string(&self, encrypted_text: &str) -> Result<String, CryptoError> {
        let encrypted_data = general_purpose::STANDARD.decode(encrypted_text)
            .map_err(|_| CryptoError::InvalidData)?;
        
        if encrypted_data.len() < 28 { // nonce(12) + tag(16)
            return Err(CryptoError::InvalidData);
        }
        
        let nonce_bytes = &encrypted_data[0..12];
        let ciphertext = &encrypted_data[12..];
        
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| CryptoError::DecryptionError)?;
        
        // 分离tag和数据
        let tag_start = ciphertext.len().saturating_sub(16);
        let data = ciphertext[..tag_start].to_vec();
        let tag_bytes = &ciphertext[tag_start..];
        
        // 解密数据（使用更简单的API）
        let in_out = data.clone();
        let _tag_array: [u8; 16] = tag_bytes.try_into().map_err(|_| CryptoError::InvalidData)?;
        
        // 使用open_in_place API（替换已废弃的open_separate_gather）
        let mut buffer = in_out;
        self.key.open_in_place(nonce, aead::Aad::empty(), &mut buffer)
            .map_err(|_| CryptoError::DecryptionError)?;
        
        String::from_utf8(buffer).map_err(|_| CryptoError::InvalidData)
    }
}

/// 全局加密管理器（单例模式）
pub struct GlobalCryptoManager {
    inner: Mutex<Option<CryptoManager>>,
}

impl GlobalCryptoManager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
    
    /// 初始化加密管理器
    pub fn initialize(&self) -> Result<(), CryptoError> {
        let mut guard = self.inner.lock().unwrap();
        if guard.is_none() {
            *guard = Some(CryptoManager::new()?);
        }
        Ok(())
    }
    
    /// 加密字符串
    pub fn encrypt_string(&self, text: &str) -> Result<String, CryptoError> {
        let guard = self.inner.lock().unwrap();
        match &*guard {
            Some(crypto) => crypto.encrypt_string(text),
            None => Err(CryptoError::EncryptionError),
        }
    }
    
    /// 解密字符串
    pub fn decrypt_string(&self, encrypted_text: &str) -> Result<String, CryptoError> {
        let guard = self.inner.lock().unwrap();
        match &*guard {
            Some(crypto) => crypto.decrypt_string(encrypted_text),
            None => Err(CryptoError::DecryptionError),
        }
    }
}

// 全局加密管理器实例
lazy_static::lazy_static! {
    pub static ref GLOBAL_CRYPTO: GlobalCryptoManager = GlobalCryptoManager::new();
}