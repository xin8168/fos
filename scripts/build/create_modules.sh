#!/bin/bash
# FOS 模块骨架生成脚本

set -e

# 定义所有新增模块
MODULES=(
    "rollback:回滚引擎"
    "permission:权限中心"
    "eventlog:事件日志"
    "notifier:通知服务"
    "bootstrap:启动引导"
    "shutdown:优雅关闭"
    "health:健康检查"
    "transaction:事务管理"
    "lock:分布式锁"
    "idempotency:幂等控制"
    "migration:数据迁移"
    "backup:数据备份"
    "ratelimiter:限流控制"
    "plugin:插件系统"
    "schedule:定时调度"
    "cache:多级缓存"
)

# 创建模块目录和骨架文件
for module_info in "${MODULES[@]}"; do
    IFS=':' read -r module desc <<< "$module_info"
    
    echo "Creating module: $module ($desc)"
    
    # 创建目录
    mkdir -p "src/$module/src"
    
    # 创建Cargo.toml
    cat > "src/$module/Cargo.toml" << EOF
[package]
name = "fos-$module"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
authors = ["FOS Team"]
description = "FOS $desc 模块"
repository = "https://github.com/fos-platform/fos"

[lib]
name = "fos_$module"
path = "src/lib.rs"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[lints]
workspace = true
EOF

    # 创建lib.rs
    cat > "src/$module/src/lib.rs" << EOF
//! # FOS ${module^} - $desc 模块
//!
//! ## 核心职责
//! - 职责待定义
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不修改指令内容

pub mod error;
pub mod config;

pub use error::{Error, Result};
pub use config::Config;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");
EOF

    # 创建error.rs
    cat > "src/$module/src/error.rs" << EOF
//! 错误类型定义

use thiserror::Error;

/// 模块错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),
    
    /// 操作错误
    #[error("操作错误: {0}")]
    Operation(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 模块Result类型
pub type Result<T> = std::result::Result<T, Error>;
EOF

    # 创建config.rs
    cat > "src/$module/src/config.rs" << EOF
//! 配置类型定义

use serde::{Deserialize, Serialize};

/// 模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 配置项（待定义）
    pub placeholder: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            placeholder: String::new(),
        }
    }
}
EOF

done

echo "All modules created successfully!"
