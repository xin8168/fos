//! 锁等待队列

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// 锁等待队列
#[derive(Debug)]
pub struct LockWaitQueue {
    /// 等待者列表
    waiters: VecDeque<Waiter>,
    /// 最大等待者数量
    max_waiters: usize,
}

/// 等待者
#[derive(Debug, Clone)]
pub struct Waiter {
    /// 等待者标识
    pub owner: String,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
    /// 是否已被通知
    pub notified: bool,
}

impl LockWaitQueue {
    /// 创建新的等待队列
    pub fn new() -> Self {
        Self { waiters: VecDeque::new(), max_waiters: 100 }
    }

    /// 设置最大等待者数量
    pub fn with_max_waiters(mut self, max: usize) -> Self {
        self.max_waiters = max;
        self
    }

    /// 添加等待者
    pub fn add_waiter(&mut self, owner: &str) -> bool {
        if self.waiters.len() >= self.max_waiters {
            return false;
        }

        // 检查是否已在队列中
        if self.waiters.iter().any(|w| w.owner == owner) {
            return false;
        }

        self.waiters.push_back(Waiter {
            owner: owner.to_string(),
            joined_at: Utc::now(),
            notified: false,
        });

        true
    }

    /// 移除等待者
    pub fn remove_waiter(&mut self, owner: &str) -> Option<Waiter> {
        if let Some(pos) = self.waiters.iter().position(|w| w.owner == owner) {
            self.waiters.remove(pos)
        } else {
            None
        }
    }

    /// 通知下一个等待者
    pub fn notify_one(&mut self) -> Option<Waiter> {
        if let Some(waiter) = self.waiters.front_mut() {
            waiter.notified = true;
            self.waiters.pop_front()
        } else {
            None
        }
    }

    /// 通知所有等待者
    pub fn notify_all(&mut self) -> Vec<Waiter> {
        let mut notified = Vec::new();
        while let Some(mut waiter) = self.waiters.pop_front() {
            waiter.notified = true;
            notified.push(waiter);
        }
        notified
    }

    /// 获取等待者数量
    pub fn len(&self) -> usize {
        self.waiters.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.waiters.is_empty()
    }

    /// 获取等待者列表
    pub fn get_waiters(&self) -> Vec<&Waiter> {
        self.waiters.iter().collect()
    }

    /// 检查指定等待者是否在队列中
    pub fn contains(&self, owner: &str) -> bool {
        self.waiters.iter().any(|w| w.owner == owner)
    }

    /// 获取等待时间最长的等待者
    pub fn peek(&self) -> Option<&Waiter> {
        self.waiters.front()
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.waiters.clear();
    }

    /// 获取队列位置
    pub fn position(&self, owner: &str) -> Option<usize> {
        self.waiters.iter().position(|w| w.owner == owner)
    }
}

impl Default for LockWaitQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_creation() {
        let queue = LockWaitQueue::new();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_add_waiter() {
        let mut queue = LockWaitQueue::new();

        assert!(queue.add_waiter("owner1"));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_duplicate_waiter() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        assert!(!queue.add_waiter("owner1")); // 重复添加失败
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_remove_waiter() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        let removed = queue.remove_waiter("owner1");

        assert!(removed.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_notify_one() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        queue.add_waiter("owner2");

        let notified = queue.notify_one();
        assert!(notified.is_some());
        assert_eq!(notified.unwrap().owner, "owner1");
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_notify_all() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        queue.add_waiter("owner2");

        let notified = queue.notify_all();
        assert_eq!(notified.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_max_waiters() {
        let mut queue = LockWaitQueue::new().with_max_waiters(2);

        assert!(queue.add_waiter("owner1"));
        assert!(queue.add_waiter("owner2"));
        assert!(!queue.add_waiter("owner3")); // 超过限制
    }

    #[test]
    fn test_contains() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        assert!(queue.contains("owner1"));
        assert!(!queue.contains("owner2"));
    }

    #[test]
    fn test_position() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        queue.add_waiter("owner2");

        assert_eq!(queue.position("owner1"), Some(0));
        assert_eq!(queue.position("owner2"), Some(1));
        assert_eq!(queue.position("owner3"), None);
    }

    #[test]
    fn test_peek() {
        let mut queue = LockWaitQueue::new();

        queue.add_waiter("owner1");
        queue.add_waiter("owner2");

        let peeked = queue.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().owner, "owner1");
        assert_eq!(queue.len(), 2); // peek不应移除
    }
}
