//! 统一的错误处理模块

use serde::{Deserialize, Serialize};
use std::fmt;

/// 应用错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    /// 配置错误
    ConfigError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 网络错误
    NetworkError(String),
    /// 认证错误
    AuthError(String),
    /// 文件系统错误
    FileSystemError(String),
    /// 加密错误
    CryptoError(String),
    /// 输入验证错误
    ValidationError(String),
    /// 未知错误
    UnknownError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AppError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            AppError::AuthError(msg) => write!(f, "认证错误: {}", msg),
            AppError::FileSystemError(msg) => write!(f, "文件系统错误: {}", msg),
            AppError::CryptoError(msg) => write!(f, "加密错误: {}", msg),
            AppError::ValidationError(msg) => write!(f, "输入验证错误: {}", msg),
            AppError::UnknownError(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileSystemError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ConfigError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::NetworkError(err.to_string())
    }
}

impl From<crate::crypto::CryptoError> for AppError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        AppError::CryptoError(err.to_string())
    }
}

/// Tauri命令错误转换器
pub trait TauriResultExt<T> {
    fn to_tauri_result(self) -> Result<T, String>;
}

impl<T> TauriResultExt<T> for AppResult<T> {
    fn to_tauri_result(self) -> Result<T, String> {
        self.map_err(|e| e.to_user_friendly())
    }
}

/// 统一的结果类型
pub type AppResult<T> = Result<T, AppError>;

/// 错误处理工具函数
pub trait ErrorExt {
    /// 转换为前端友好的错误消息
    fn to_user_friendly(&self) -> String;
    
    /// 记录错误日志
    fn log_error(&self, context: &str);
}

impl ErrorExt for AppError {
    fn to_user_friendly(&self) -> String {
        match self {
            AppError::ConfigError(_) => "配置错误，请检查设置".to_string(),
            AppError::DatabaseError(_) => "数据库操作失败".to_string(),
            AppError::NetworkError(_) => "网络连接失败，请检查网络设置".to_string(),
            AppError::AuthError(_) => "认证失败，请检查API密钥".to_string(),
            AppError::FileSystemError(_) => "文件操作失败".to_string(),
            AppError::CryptoError(_) => "加密操作失败".to_string(),
            AppError::ValidationError(msg) => format!("输入验证失败: {}", msg),
            AppError::UnknownError(_) => "系统内部错误".to_string(),
        }
    }
    
    fn log_error(&self, context: &str) {
        log::error!("{}: {}", context, self);
    }
}

/// 安全的错误返回宏
#[macro_export]
macro_rules! safe_return {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                let app_error: AppError = e.into();
                app_error.log_error($context);
                return Err(app_error);
            }
        }
    };
}

/// 安全的数据库操作宏
#[macro_export]
macro_rules! safe_db {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                log::error!("数据库操作失败 [{}]: {}", $operation, e);
                return Err(AppError::DatabaseError(format!("{}: {}", $operation, e)));
            }
        }
    };
}