//! 稳定性测试工具和配置

use std::time::Duration;

/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试名称
    pub name: String,
    /// 测试持续时间
    pub duration: Duration,
    /// 最大并发数
    pub max_concurrent: usize,
    /// 操作间隔
    pub operation_interval: Duration,
    /// 是否启用内存监控
    pub enable_memory_monitoring: bool,
    /// 是否启用CPU监控
    pub enable_cpu_monitoring: bool,
}

impl TestConfig {
    const fn new_default() -> Self {
        Self {
            name: String::new(),
            duration: Duration::from_secs(10), // 默认10秒
            max_concurrent: 10,
            operation_interval: Duration::from_millis(10),
            enable_memory_monitoring: true,
            enable_cpu_monitoring: true,
        }
    }

    /// 创建新的测试配置
    pub fn new() -> Self {
        Self::new_default()
    }

    /// 设置测试名称
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// 设置测试持续时间
    pub const fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// 设置最大并发数
    pub const fn with_max_concurrent(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent = max_concurrent;
        self
    }

    /// 设置操作间隔
    pub const fn with_operation_interval(mut self, interval: Duration) -> Self {
        self.operation_interval = interval;
        self
    }

    /// 启用内存监控
    pub const fn with_memory_monitoring(mut self, enable: bool) -> Self {
        self.enable_memory_monitoring = enable;
        self
    }

    /// 启用CPU监控
    pub const fn with_cpu_monitoring(mut self, enable: bool) -> Self {
        self.enable_cpu_monitoring = enable;
        self
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new_default()
    }
}

/// 测试结果
#[derive(Debug, Clone)]
pub struct TestResult {
    /// 测试名称
    pub test_name: String,
    /// 是否通过
    pub passed: bool,
    /// 总操作数
    pub total_operations: u64,
    /// 成功操作数
    pub successful_operations: u64,
    /// 失败操作数
    pub failed_operations: u64,
    /// 平均操作延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 最大操作延迟（毫秒）
    pub max_latency_ms: f64,
    /// 吞吐量（操作/秒）
    pub throughput: f64,
    /// 指标数据
    pub metrics: Metrics,
}

impl TestResult {
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            passed: false,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0.0,
            throughput: 0.0,
            metrics: Metrics::default(),
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        (self.successful_operations as f64 / self.total_operations as f64) * 100.0
    }

    /// 设置通过状态
    pub fn with_passed(mut self, passed: bool) -> Self {
        self.passed = passed;
        self
    }

    /// 设置操作计数
    pub fn with_operations(mut self, total: u64, successful: u64, failed: u64) -> Self {
        self.total_operations = total;
        self.successful_operations = successful;
        self.failed_operations = failed;
        self
    }

    /// 设置延迟数据
    pub fn with_latency(mut self, avg_ms: f64, max_ms: f64) -> Self {
        self.avg_latency_ms = avg_ms;
        self.max_latency_ms = max_ms;
        self
    }

    /// 设置吞吐量
    pub fn with_throughput(mut self, throughput: f64) -> Self {
        self.throughput = throughput;
        self
    }

    /// 设置指标
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
}

/// 性能指标
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// 内存使用量（字节）
    pub memory_usage_bytes: u64,
    /// 最大内存使用量（字节）
    pub peak_memory_bytes: u64,
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    /// 最大CPU使用率（百分比）
    pub peak_cpu_percent: f64,
    /// 线程数
    pub thread_count: usize,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新峰值内存
    pub fn update_peak_memory(&mut self) {
        if self.memory_usage_bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = self.memory_usage_bytes;
        }
    }

    /// 更新峰值CPU
    pub fn update_peak_cpu(&mut self) {
        if self.cpu_usage_percent > self.peak_cpu_percent {
            self.peak_cpu_percent = self.cpu_usage_percent;
        }
    }
}
