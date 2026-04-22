//! 错误类型定义

use thiserror::Error;

/// 迁移错误
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("迁移已注册: {0}")]
    AlreadyRegistered(String),

    #[error("依赖不存在: {0}")]
    DependencyNotFound(String),

    #[error("版本未找到: {0}")]
    VersionNotFound(String),

    #[error("版本已迁移: {0}")]
    AlreadyMigrated(String),

    #[error("无法回滚：版本 {0} 不支持或未迁移")]
    CannotRollback(String),

    #[error("非法的版本方向：目标版本必须小于当前版本")]
    InvalidVersionDirection,

    #[error("没有可回滚的迁移")]
    NoMigration,

    #[error("迁移执行失败: {0}")]
    ExecutionFailed(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 迁移结果类型
pub type Result<T> = std::result::Result<T, MigrationError>;
