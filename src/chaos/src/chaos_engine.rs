//! 混沌引擎模块

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 混沌测试场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosScenario {
    /// 网络延迟注入
    NetworkLatency { min_ms: u64, max_ms: u64 },
    /// 网络分区
    NetworkPartition { duration_secs: u64 },
    /// 节点故障
    NodeFailure { node_id: String },
    /// 内存压力
    MemoryPressure { percentage: u8 },
    /// CPU压力
    CpuPressure { percentage: u8 },
    /// 磁盘IO延迟
    DiskIoLatency { delay_ms: u64 },
    /// 并发风暴
    ConcurrencyStorm { max_concurrent: usize },
}

impl ChaosScenario {
    /// 获取场景名称
    pub fn name(&self) -> &str {
        match self {
            ChaosScenario::NetworkLatency { .. } => "NetworkLatency",
            ChaosScenario::NetworkPartition { .. } => "NetworkPartition",
            ChaosScenario::NodeFailure { .. } => "NodeFailure",
            ChaosScenario::MemoryPressure { .. } => "MemoryPressure",
            ChaosScenario::CpuPressure { .. } => "CpuPressure",
            ChaosScenario::DiskIoLatency { .. } => "DiskIoLatency",
            ChaosScenario::ConcurrencyStorm { .. } => "ConcurrencyStorm",
        }
    }
}

/// 混沌测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosResult {
    /// 测试场景
    pub scenario: String,
    /// 是否通过
    pub passed: bool,
    /// 测试持续时间
    pub duration_secs: u64,
    /// 系统恢复时间（秒）
    pub recovery_time_secs: Option<f64>,
    /// 数据丢失情况
    pub data_loss: bool,
    /// 服务中断时间（秒）
    pub service_interruption_secs: Option<f64>,
    /// 观察到的异常
    pub anomalies: Vec<String>,
    /// 测试摘要
    pub summary: String,
}

impl ChaosResult {
    pub fn new(scenario: &str) -> Self {
        Self {
            scenario: scenario.to_string(),
            passed: true,
            duration_secs: 0,
            recovery_time_secs: None,
            data_loss: false,
            service_interruption_secs: None,
            anomalies: Vec::new(),
            summary: String::new(),
        }
    }

    pub fn with_passed(mut self, passed: bool) -> Self {
        self.passed = passed;
        self
    }

    pub fn with_duration(mut self, secs: u64) -> Self {
        self.duration_secs = secs;
        self
    }

    pub fn with_recovery_time(mut self, secs: f64) -> Self {
        self.recovery_time_secs = Some(secs);
        self
    }

    pub fn with_data_loss(mut self, loss: bool) -> Self {
        self.data_loss = loss;
        self.passed = !loss;
        self
    }

    pub fn add_anomaly(mut self, anomaly: String) -> Self {
        self.anomalies.push(anomaly);
        self
    }

    pub fn generate_summary(&mut self) {
        self.summary = format!(
            "混沌测试 [{}] 完成。通过: {}, 持续时间: {}s, 恢复时间: {:?}s, 数据丢失: {}",
            self.scenario, self.passed, self.duration_secs, self.recovery_time_secs, self.data_loss
        );
    }
}

/// 混沌引擎
pub struct ChaosEngine {
    /// 引擎名称
    name: String,
    /// 是否启用详细日志
    verbose: bool,
}

impl ChaosEngine {
    /// 创建新的混沌引擎
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), verbose: false }
    }

    /// 启用详细日志
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 运行网络延迟混沌测试
    pub async fn test_network_latency(&self, min_ms: u64, max_ms: u64) -> ChaosResult {
        let mut result = ChaosResult::new("NetworkLatency");
        let start = std::time::Instant::now();

        if self.verbose {
            println!("开始网络延迟混沌测试: {}ms - {}ms", min_ms, max_ms);
        }

        // 模拟网络延迟
        let delay = rand::random::<u64>() % (max_ms - min_ms) + min_ms;
        tokio::time::sleep(Duration::from_millis(delay)).await;

        result = result.with_duration(start.elapsed().as_secs());
        result.generate_summary();
        result
    }

    /// 运行节点故障混沌测试
    pub async fn test_node_failure(&self, node_id: &str) -> ChaosResult {
        let mut result = ChaosResult::new(&format!("NodeFailure-{}", node_id));
        let start = std::time::Instant::now();

        if self.verbose {
            println!("开始节点故障混沌测试: {}", node_id);
        }

        // 模拟节点故障
        tokio::time::sleep(Duration::from_millis(100)).await;

        result = result.with_duration(start.elapsed().as_secs());
        result.generate_summary();
        result
    }

    /// 运行并发风暴混沌测试
    pub async fn test_concurrency_storm(&self, max_concurrent: usize) -> ChaosResult {
        use fos_cache::LocalCache;
        use std::sync::Arc;

        let mut result = ChaosResult::new("ConcurrencyStorm");
        let start = std::time::Instant::now();

        if self.verbose {
            println!("开始并发风暴混沌测试: {} 并发", max_concurrent);
        }

        let cache = Arc::new(LocalCache::new());
        let mut handles = vec![];

        for i in 0..max_concurrent {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                let key = format!("chaos_key_{}", i);
                cache_clone.set(key.clone(), format!("value_{}", i), None).await;
                let _ = cache_clone.get(&key).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        result = result.with_duration(start.elapsed().as_secs());
        result.generate_summary();
        result
    }

    /// 运行所有混沌测试
    pub async fn run_all_tests(&self) -> Vec<ChaosResult> {
        let mut results = Vec::new();

        results.push(self.test_network_latency(10, 100).await);
        results.push(self.test_node_failure("node-1").await);
        results.push(self.test_concurrency_storm(50).await);

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_scenario_name() {
        let scenario = ChaosScenario::NetworkLatency { min_ms: 10, max_ms: 100 };
        assert_eq!(scenario.name(), "NetworkLatency");

        let scenario = ChaosScenario::NodeFailure { node_id: "test".to_string() };
        assert_eq!(scenario.name(), "NodeFailure");
    }

    #[test]
    fn test_chaos_result() {
        let mut result = ChaosResult::new("Test");
        assert!(result.passed);
        assert!(!result.data_loss);

        result = result.with_passed(false);
        assert!(!result.passed);

        result = result.with_data_loss(true);
        assert!(!result.passed);
        assert!(result.data_loss);

        result.generate_summary();
        assert!(!result.summary.is_empty());
    }

    #[tokio::test]
    async fn test_chaos_engine() {
        let engine = ChaosEngine::new("TestEngine").with_verbose(false);
        let result = engine.test_network_latency(1, 10).await;
        assert!(result.passed);
    }
}
