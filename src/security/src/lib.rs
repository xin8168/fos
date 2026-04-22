//! FOS 安全测试模块
//!
//! 提供安全渗透测试和漏洞检测功能

pub mod penetration;
pub mod vulnerability;

pub use penetration::PenetrationTest;
pub use penetration::PenetrationTestResult;
