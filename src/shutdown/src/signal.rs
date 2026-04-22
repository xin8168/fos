//! 信号处理器

use crate::error::Result;
use std::sync::atomic::{AtomicBool, Ordering};

/// 关闭信号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSignal {
    /// SIGINT (Ctrl+C)
    Interrupt,
    /// SIGTERM
    Terminate,
    /// SIGQUIT
    Quit,
    /// 用户自定义
    Custom,
}

/// 信号处理器
pub struct SignalHandler {
    /// 收到信号标志
    received: AtomicBool,
    /// 收到的信号类型
    signal_type: std::sync::Mutex<Option<ShutdownSignal>>,
}

impl SignalHandler {
    /// 创建新的信号处理器
    pub fn new() -> Self {
        Self { received: AtomicBool::new(false), signal_type: std::sync::Mutex::new(None) }
    }

    /// 检查是否收到关闭信号
    pub fn is_shutdown_requested(&self) -> bool {
        self.received.load(Ordering::SeqCst)
    }

    /// 获取收到的信号类型
    pub fn get_signal(&self) -> Option<ShutdownSignal> {
        self.signal_type.lock().unwrap().clone()
    }

    /// 模拟接收信号（用于测试）
    pub fn receive_signal(&self, signal: ShutdownSignal) {
        self.received.store(true, Ordering::SeqCst);
        *self.signal_type.lock().unwrap() = Some(signal);
    }

    /// 重置信号状态
    pub fn reset(&self) {
        self.received.store(false, Ordering::SeqCst);
        *self.signal_type.lock().unwrap() = None;
    }

    /// 等待信号
    pub async fn wait_for_signal(&self) -> Result<ShutdownSignal> {
        // 简单轮询实现
        loop {
            if self.is_shutdown_requested() {
                return Ok(self.get_signal().unwrap_or(ShutdownSignal::Custom));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(!handler.is_shutdown_requested());
    }

    #[test]
    fn test_receive_signal() {
        let handler = SignalHandler::new();
        handler.receive_signal(ShutdownSignal::Interrupt);

        assert!(handler.is_shutdown_requested());
        assert_eq!(handler.get_signal(), Some(ShutdownSignal::Interrupt));
    }

    #[test]
    fn test_reset() {
        let handler = SignalHandler::new();
        handler.receive_signal(ShutdownSignal::Terminate);
        handler.reset();

        assert!(!handler.is_shutdown_requested());
    }
}
