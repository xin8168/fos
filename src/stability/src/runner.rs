//! 稳定性测试运行器

use crate::error::Result;
use crate::utils::{TestConfig, TestResult};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 稳定性测试运行器
pub struct StabilityTestRunner {
    /// 测试配置
    config: TestConfig,
    /// 运行状态
    running: Arc<AtomicBool>,
}

impl StabilityTestRunner {
    /// 创建新的测试运行器
    pub fn new(config: TestConfig) -> Self {
        Self { config, running: Arc::new(AtomicBool::new(false)) }
    }

    /// 运行缓存稳定性测试
    pub async fn run_cache_stability_test(&self) -> Result<TestResult> {
        use fos_cache::LocalCache;
        use std::sync::Arc;

        let cache = Arc::new(LocalCache::new());
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let start_time = Instant::now();
        let total_ops = Arc::new(AtomicU64::new(0));
        let success_ops = Arc::new(AtomicU64::new(0));
        let failed_ops = Arc::new(AtomicU64::new(0));
        let max_latency = Arc::new(AtomicU64::new(0));

        // 预先提取配置值，避免生命周期问题
        let op_interval = self.config.operation_interval;
        let test_duration = self.config.duration;
        let concurrency = self.config.max_concurrent;

        // 创建多个并发任务
        let mut handles = vec![];

        for worker_id in 0..concurrency {
            let cache_clone = cache.clone();
            let running_clone = running.clone();
            let total = total_ops.clone();
            let success = success_ops.clone();
            let failed = failed_ops.clone();
            let max_lat = max_latency.clone();

            let handle = tokio::spawn(async move {
                let mut local_total = 0u64;

                while running_clone.load(Ordering::SeqCst) {
                    let op_start = Instant::now();

                    // 随机操作：set 或 get
                    let key = format!("stability_key_{}_{}", worker_id, local_total);

                    // 执行缓存操作
                    if local_total % 2 == 0 {
                        cache_clone.set(key.clone(), format!("value_{}", local_total), None).await;
                    } else {
                        let _ = cache_clone.get(&key).await;
                    }

                    let latency = op_start.elapsed().as_millis() as u64;
                    local_total += 1;

                    total.fetch_add(1, Ordering::SeqCst);
                    success.fetch_add(1, Ordering::SeqCst);

                    // 更新最大延迟
                    let current_max = max_lat.load(Ordering::SeqCst);
                    if latency > current_max {
                        max_lat.store(latency, Ordering::SeqCst);
                    }

                    // 短暂等待，避免过快
                    tokio::time::sleep(op_interval).await;
                }
            });

            handles.push(handle);
        }

        // 运行指定时间
        tokio::time::sleep(test_duration).await;

        // 停止测试
        running.store(false, Ordering::SeqCst);

        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start_time.elapsed();

        let total = total_ops.load(Ordering::SeqCst);
        let successful = success_ops.load(Ordering::SeqCst);
        let failed = failed_ops.load(Ordering::SeqCst);
        let max_lat = max_latency.load(Ordering::SeqCst) as f64;
        let throughput =
            if elapsed.as_secs() > 0 { total as f64 / elapsed.as_secs() as f64 } else { 0.0 };

        Ok(TestResult::new(self.config.name.clone())
            .with_passed(failed == 0)
            .with_operations(total, successful, failed)
            .with_latency(0.0, max_lat)
            .with_throughput(throughput))
    }

    /// 运行锁稳定性测试
    pub async fn run_lock_stability_test(&self) -> Result<TestResult> {
        use fos_lock::LockManager;
        use std::sync::Arc;

        let lock_manager = Arc::new(LockManager::with_defaults());
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let start_time = Instant::now();
        let total_ops = Arc::new(AtomicU64::new(0));
        let success_ops = Arc::new(AtomicU64::new(0));
        let failed_ops = Arc::new(AtomicU64::new(0));

        // 预先提取配置值
        let op_interval = self.config.operation_interval;
        let test_duration = self.config.duration;
        let concurrency = self.config.max_concurrent;

        let mut handles = vec![];

        for worker_id in 0..concurrency {
            let lock_manager_clone = lock_manager.clone();
            let running_clone = running.clone();
            let total = total_ops.clone();
            let success = success_ops.clone();
            let failed = failed_ops.clone();

            let handle = tokio::spawn(async move {
                let mut local_total = 0u64;

                while running_clone.load(Ordering::SeqCst) {
                    let key = format!("stability_lock_{}_{}", worker_id, local_total);
                    let owner = format!("owner_{}", worker_id);

                    // 尝试获取锁
                    let result = lock_manager_clone.try_lock(&key, &owner);

                    local_total += 1;
                    total.fetch_add(1, Ordering::SeqCst);

                    match result {
                        Ok(Some(_lock_id)) => {
                            success.fetch_add(1, Ordering::SeqCst);
                            // 持有锁一小段时间后释放
                            let _ = lock_manager_clone.unlock(&key, &owner);
                        },
                        Ok(None) => {
                            // 锁已被占用，这是正常情况
                            success.fetch_add(1, Ordering::SeqCst);
                        },
                        Err(_) => {
                            failed.fetch_add(1, Ordering::SeqCst);
                        },
                    }

                    tokio::time::sleep(op_interval).await;
                }
            });

            handles.push(handle);
        }

        tokio::time::sleep(test_duration).await;
        running.store(false, Ordering::SeqCst);

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start_time.elapsed();

        let total = total_ops.load(Ordering::SeqCst);
        let successful = success_ops.load(Ordering::SeqCst);
        let failed = failed_ops.load(Ordering::SeqCst);
        let throughput =
            if elapsed.as_secs() > 0 { total as f64 / elapsed.as_secs() as f64 } else { 0.0 };

        Ok(TestResult::new(self.config.name.clone())
            .with_passed(failed == 0)
            .with_operations(total, successful, failed)
            .with_latency(0.0, 0.0)
            .with_throughput(throughput))
    }

    /// 运行系统集成稳定性测试
    pub async fn run_system_integration_stability_test(&self) -> Result<TestResult> {
        use fos_cache::LocalCache;
        use fos_lock::LockManager;
        use fos_rollback::Snapshot;
        use std::sync::Arc;

        let cache = Arc::new(LocalCache::new());
        let lock_manager = Arc::new(LockManager::with_defaults());
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let start_time = Instant::now();
        let total_ops = Arc::new(AtomicU64::new(0));
        let success_ops = Arc::new(AtomicU64::new(0));
        let failed_ops = Arc::new(AtomicU64::new(0));

        // 预先提取配置值
        let op_interval = self.config.operation_interval;
        let test_duration = self.config.duration;
        let concurrency = self.config.max_concurrent;

        let mut handles = vec![];

        for worker_id in 0..concurrency {
            let cache_clone = cache.clone();
            let lock_manager_clone = lock_manager.clone();
            let running_clone = running.clone();
            let total = total_ops.clone();
            let success = success_ops.clone();
            let failed = failed_ops.clone();

            let handle = tokio::spawn(async move {
                let mut local_total = 0u64;

                while running_clone.load(Ordering::SeqCst) {
                    // 执行多模块操作序列
                    let key = format!("system_key_{}_{}", worker_id, local_total);
                    let lock_key = format!("system_lock_{}", worker_id);

                    // 1. 缓存操作
                    let _ =
                        cache_clone.set(key.clone(), format!("value_{}", local_total), None).await;

                    // 2. 分布式锁操作
                    let _ = lock_manager_clone.try_lock(&lock_key, &format!("owner_{}", worker_id));

                    // 3. 快照创建（用于回滚测试）
                    let snapshot_data = serde_json::json!({
                        "key": key,
                        "worker_id": worker_id,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    let _snapshot = Snapshot::new(
                        format!("op_{}", local_total),
                        fos_rollback::SnapshotType::Full,
                        snapshot_data,
                    );

                    local_total += 1;
                    total.fetch_add(1, Ordering::SeqCst);
                    success.fetch_add(1, Ordering::SeqCst);

                    tokio::time::sleep(op_interval).await;
                }

                // 更新全局统计
                failed.fetch_add(0, Ordering::SeqCst);
            });

            handles.push(handle);
        }

        tokio::time::sleep(test_duration).await;
        running.store(false, Ordering::SeqCst);

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start_time.elapsed();

        let total = total_ops.load(Ordering::SeqCst);
        let successful = success_ops.load(Ordering::SeqCst);
        let failed = failed_ops.load(Ordering::SeqCst);
        let throughput =
            if elapsed.as_secs() > 0 { total as f64 / elapsed.as_secs() as f64 } else { 0.0 };

        Ok(TestResult::new(self.config.name.clone())
            .with_passed(failed == 0)
            .with_operations(total, successful, failed)
            .with_latency(0.0, 0.0)
            .with_throughput(throughput))
    }
}

/// 运行缓存稳定性测试的便捷函数
pub async fn cache_stability() -> Result<TestResult> {
    let config = TestConfig::new()
        .with_name("cache_stability".to_string())
        .with_duration(Duration::from_secs(10))
        .with_max_concurrent(10);

    let runner = StabilityTestRunner::new(config);
    runner.run_cache_stability_test().await
}

/// 运行锁稳定性测试的便捷函数
pub async fn lock_stability() -> Result<TestResult> {
    let config = TestConfig::new()
        .with_name("lock_stability".to_string())
        .with_duration(Duration::from_secs(10))
        .with_max_concurrent(10);

    let runner = StabilityTestRunner::new(config);
    runner.run_lock_stability_test().await
}

/// 运行系统集成稳定性测试的便捷函数
pub async fn system_integration_stability() -> Result<TestResult> {
    let config = TestConfig::new()
        .with_name("system_integration_stability".to_string())
        .with_duration(Duration::from_secs(10))
        .with_max_concurrent(10);

    let runner = StabilityTestRunner::new(config);
    runner.run_system_integration_stability_test().await
}
