//! # 缓存统计信息
//!
//! 缓存性能统计

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 驱逐次数
    pub evictions: u64,
    /// 当前条目数
    pub size: usize,
}

impl CacheStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录命中
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// 记录未命中
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// 记录驱逐
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// 设置大小
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64)
        }
    }

    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_record_hit() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.total_requests(), 2);
    }

    #[test]
    fn test_record_miss() {
        let mut stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.misses, 3);
        assert_eq!(stats.total_requests(), 3);
    }

    #[test]
    fn test_record_eviction() {
        let mut stats = CacheStats::new();
        stats.record_eviction();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_hit_rate() {
        let mut stats = CacheStats::new();

        // 无请求时的命中率应为0
        assert_eq!(stats.hit_rate(), 0.0);

        // 50%命中率
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.5);

        // 75%命中率
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hit_rate(), 0.75);
    }

    #[test]
    fn test_total_requests() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.record_hit();
        assert_eq!(stats.total_requests(), 3);
    }

    #[test]
    fn test_set_size() {
        let mut stats = CacheStats::new();
        stats.set_size(100);
        assert_eq!(stats.size, 100);

        stats.set_size(50);
        assert_eq!(stats.size, 50);
    }

    #[test]
    fn test_reset() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        stats.set_size(100);

        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.evictions, 1);
        assert_eq!(stats.size, 100);

        stats.reset();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0); // reset会将所有统计重置包括size
    }
}
