//! 资源清理器

use crate::error::Result;
use std::collections::VecDeque;

/// 清理操作
pub type CleanupAction = Box<dyn Fn() -> Result<()> + Send + Sync>;

/// 清理项
pub struct CleanupItem {
    /// 名称
    pub name: String,
    /// 优先级（数字越大越先执行）
    pub priority: u32,
    /// 清理操作
    pub action: CleanupAction,
}

/// 资源清理器
pub struct ResourceCleaner {
    /// 清理项列表
    cleanup_items: std::sync::Mutex<VecDeque<CleanupItem>>,
    /// 清理状态
    cleaned: std::sync::atomic::AtomicBool,
}

impl ResourceCleaner {
    /// 创建新的资源清理器
    pub fn new() -> Self {
        Self {
            cleanup_items: std::sync::Mutex::new(VecDeque::new()),
            cleaned: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 注册清理项
    pub fn register<F>(&self, name: &str, priority: u32, action: F)
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        let item = CleanupItem { name: name.to_string(), priority, action: Box::new(action) };
        self.cleanup_items.lock().unwrap().push_back(item);
    }

    /// 执行所有清理
    pub fn cleanup_all(&self) -> Result<()> {
        if self.cleaned.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        let mut items = self.cleanup_items.lock().unwrap();

        // 按优先级排序（降序）
        items.make_contiguous().sort_by(|a, b| b.priority.cmp(&a.priority));

        while let Some(item) = items.pop_front() {
            tracing::info!("执行清理: {}", item.name);

            if let Err(e) = (item.action)() {
                tracing::error!("清理失败 {}: {}", item.name, e);
            }
        }

        self.cleaned.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 检查是否已清理
    pub fn is_cleaned(&self) -> bool {
        self.cleaned.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 获取待清理项数量
    pub fn pending_count(&self) -> usize {
        self.cleanup_items.lock().unwrap().len()
    }

    /// 清空清理列表
    pub fn clear(&self) {
        self.cleanup_items.lock().unwrap().clear();
    }
}

impl Default for ResourceCleaner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_creation() {
        let cleaner = ResourceCleaner::new();
        assert_eq!(cleaner.pending_count(), 0);
    }

    #[test]
    fn test_register_cleanup() {
        let cleaner = ResourceCleaner::new();
        cleaner.register("test", 1, || Ok(()));

        assert_eq!(cleaner.pending_count(), 1);
    }

    #[test]
    fn test_cleanup_all() {
        let cleaner = ResourceCleaner::new();
        cleaner.register("test1", 2, || Ok(()));
        cleaner.register("test2", 1, || Ok(()));

        cleaner.cleanup_all().unwrap();

        assert!(cleaner.is_cleaned());
        assert_eq!(cleaner.pending_count(), 0);
    }
}
