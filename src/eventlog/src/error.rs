//! 错误类型定义

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("事件记录错误: {0}")]
    Event(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
