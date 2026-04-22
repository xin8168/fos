//! # FOS Common - 共享类型库
//!
//! FOS 项目的核心数据结构和枚举，供所有模块共享使用
//!
//! ## 核心类型
//! - `SixAnchor` - FOS 6维锚定结构
//! - `FourElement` - FOS 4要素执行结构
//! - `FosCommand` - FOS 完整命令结构
//! - `FosEvent` - FOS 执行事件
//! - `EventType` - 事件类型枚举
//! - `EventStatus` - 事件状态枚举
//! - `ExecutionResult` - 执行结果
//! - `StepResult` - 步骤执行结果
//! - `FosError` - 全局统一错误类型

pub mod anchor;
pub mod command;
pub mod element;
pub mod error;
pub mod event;
pub mod message;
pub mod result;
pub mod status;

pub use anchor::SixAnchor;
pub use command::FosCommand;
pub use element::FourElement;
pub use error::{Error, ErrorKind, Result};
pub use event::{EventMetadata, FosEvent};
pub use message::{
    CommandRequest, CommandResponse, ExecutionRequest, ExecutionResponse, FosMessage,
    HeartbeatMessage, MessageRouter, ValidationRequest, ValidationResponse,
};
pub use result::{ExecutionResult, StepResult};
pub use status::{EventStatus, EventType};

/// FOS 项目版本号
pub const FOS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// FOS 协议版本
pub const PROTOCOL_VERSION: &str = "1.0.0";
