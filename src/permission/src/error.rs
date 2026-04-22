//! 错误类型定义

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("权限验证错误: {0}")]
    Permission(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
