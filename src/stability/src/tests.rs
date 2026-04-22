//! 稳定性测试模块

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{cache_stability, lock_stability, system_integration_stability};
    use std::time::Duration;

    /// 缓存稳定性测试
    #[tokio::test]
    async fn test_cache_stability_basic() {
        println!("开始缓存稳定性测试...");

        let result = cache_stability().await;

        match result {
            Ok(test_result) => {
                println!(
                    "缓存稳定性测试完成: 总操作={}, 成功={}, 失败={}, 吞吐量={:.2} ops/s",
                    test_result.total_operations,
                    test_result.successful_operations,
                    test_result.failed_operations,
                    test_result.throughput
                );

                // 验证大部分操作成功
                let success_rate = test_result.success_rate();
                assert!(success_rate >= 95.0, "成功率应该 >= 95%, 实际: {:.2}%", success_rate);
            },
            Err(e) => {
                panic!("缓存稳定性测试失败: {:?}", e);
            },
        }
    }

    /// 锁稳定性测试
    #[tokio::test]
    async fn test_lock_stability_basic() {
        println!("开始锁稳定性测试...");

        let result = lock_stability().await;

        match result {
            Ok(test_result) => {
                println!(
                    "锁稳定性测试完成: 总操作={}, 成功={}, 失败={}, 吞吐量={:.2} ops/s",
                    test_result.total_operations,
                    test_result.successful_operations,
                    test_result.failed_operations,
                    test_result.throughput
                );

                // 验证大部分操作成功
                let success_rate = test_result.success_rate();
                assert!(success_rate >= 95.0, "成功率应该 >= 95%, 实际: {:.2}%", success_rate);
            },
            Err(e) => {
                panic!("锁稳定性测试失败: {:?}", e);
            },
        }
    }

    /// 系统集成稳定性测试
    #[tokio::test]
    async fn test_system_integration_stability() {
        println!("开始系统集成稳定性测试...");

        let result = system_integration_stability().await;

        match result {
            Ok(test_result) => {
                println!(
                    "系统集成稳定性测试完成: 总操作={}, 成功={}, 失败={}, 吞吐量={:.2} ops/s",
                    test_result.total_operations,
                    test_result.successful_operations,
                    test_result.failed_operations,
                    test_result.throughput
                );

                // 验证大部分操作成功
                let success_rate = test_result.success_rate();
                assert!(success_rate >= 90.0, "成功率应该 >= 90%, 实际: {:.2}%", success_rate);
            },
            Err(e) => {
                panic!("系统集成稳定性测试失败: {:?}", e);
            },
        }
    }

    /// 并发压力测试
    #[tokio::test]
    async fn test_high_concurrency_stability() {
        use crate::runner::StabilityTestRunner;
        use crate::utils::TestConfig;

        println!("开始高并发压力测试...");

        let config = TestConfig::new()
            .with_name("high_concurrency_test".to_string())
            .with_duration(Duration::from_secs(5))
            .with_max_concurrent(50); // 更高的并发数

        let runner = StabilityTestRunner::new(config);
        let result = runner.run_cache_stability_test().await;

        match result {
            Ok(test_result) => {
                println!(
                    "高并发压力测试完成: 总操作={}, 成功={}, 失败={}, 吞吐量={:.2} ops/s",
                    test_result.total_operations,
                    test_result.successful_operations,
                    test_result.failed_operations,
                    test_result.throughput
                );

                // 验证系统在高并发下仍然稳定
                let success_rate = test_result.success_rate();
                assert!(
                    success_rate >= 80.0,
                    "高并发下成功率应该 >= 80%, 实际: {:.2}%",
                    success_rate
                );
            },
            Err(e) => {
                panic!("高并发压力测试失败: {:?}", e);
            },
        }
    }

    /// 长时间运行稳定性测试
    #[tokio::test]
    #[ignore] // 长时间运行，默认跳过
    async fn test_long_running_stability() {
        use crate::runner::StabilityTestRunner;
        use crate::utils::TestConfig;

        println!("开始长时间运行稳定性测试（60秒）...");

        let config = TestConfig::new()
            .with_name("long_running_test".to_string())
            .with_duration(Duration::from_secs(60)) // 运行60秒
            .with_max_concurrent(20);

        let runner = StabilityTestRunner::new(config);
        let result = runner.run_system_integration_stability_test().await;

        match result {
            Ok(test_result) => {
                println!(
                    "长时间运行稳定性测试完成: 总操作={}, 成功={}, 失败={}, 吞吐量={:.2} ops/s",
                    test_result.total_operations,
                    test_result.successful_operations,
                    test_result.failed_operations,
                    test_result.throughput
                );

                let success_rate = test_result.success_rate();
                assert!(
                    success_rate >= 95.0,
                    "长时间运行成功率应该 >= 95%, 实际: {:.2}%",
                    success_rate
                );
            },
            Err(e) => {
                panic!("长时间运行稳定性测试失败: {:?}", e);
            },
        }
    }
}
