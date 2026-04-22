//! 配置监听器

use crate::error::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// 配置变更回调
pub type ConfigChangeCallback = Arc<dyn Fn() -> Result<()> + Send + Sync>;

/// 配置监听器
pub struct ConfigWatcher {
    /// 监听路径
    path: Option<PathBuf>,
    /// 轮询间隔
    poll_interval: Duration,
    /// 是否正在监听
    watching: std::sync::atomic::AtomicBool,
    /// 变更回调
    on_change: Option<ConfigChangeCallback>,
}

impl ConfigWatcher {
    /// 创建新的配置监听器
    pub fn new() -> Self {
        Self {
            path: None,
            poll_interval: Duration::from_secs(5),
            watching: std::sync::atomic::AtomicBool::new(false),
            on_change: None,
        }
    }

    /// 设置监听路径
    pub fn watch_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 设置轮询间隔
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// 设置变更回调
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }

    /// 开始监听（同步版本，用于测试）
    pub fn watch(&self) -> Result<()> {
        self.watching.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 停止监听
    pub fn stop(&self) {
        self.watching.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// 检查是否正在监听
    pub fn is_watching(&self) -> bool {
        self.watching.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 获取监听路径
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// 获取轮询间隔
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// 触发变更回调
    pub fn trigger_change(&self) -> Result<()> {
        if let Some(ref callback) = self.on_change {
            callback()?;
        }
        Ok(())
    }

    /// 异步监听（实际实现）
    #[cfg(feature = "async")]
    pub async fn watch_async(&self) -> Result<()> {
        use tokio::time::{sleep, Instant};

        if self.path.is_none() {
            return Err(crate::error::ConfigError::ValidationError("监听路径未设置".to_string()));
        }

        let path = self.path.as_ref().unwrap().clone();
        let mut last_modified = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());

        self.watching.store(true, std::sync::atomic::Ordering::SeqCst);

        while self.is_watching() {
            sleep(self.poll_interval).await;

            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let changed = last_modified.map_or(true, |last| modified > last);

                    if changed {
                        last_modified = Some(modified);
                        self.trigger_change()?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        let watcher = ConfigWatcher::new();
        assert!(!watcher.is_watching());
    }

    #[test]
    fn test_start_stop() {
        let watcher = ConfigWatcher::new();
        watcher.watch().unwrap();
        assert!(watcher.is_watching());

        watcher.stop();
        assert!(!watcher.is_watching());
    }

    #[test]
    fn test_watch_path() {
        let watcher = ConfigWatcher::new().watch_path("config.yaml");
        assert_eq!(watcher.path().unwrap().to_str().unwrap(), "config.yaml");
    }

    #[test]
    fn test_poll_interval() {
        let watcher = ConfigWatcher::new().with_interval(Duration::from_secs(10));
        assert_eq!(watcher.poll_interval(), Duration::from_secs(10));
    }

    #[test]
    fn test_on_change_callback() {
        use std::sync::atomic::AtomicU32;
        use std::sync::Arc;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let watcher = ConfigWatcher::new().on_change(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        watcher.trigger_change().unwrap();
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
