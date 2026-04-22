//! 测试辅助函数

use std::time::Duration;
use tokio::time::sleep;

/// 等待指定毫秒数
pub async fn wait_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

/// 等待指定秒数
pub async fn wait_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

/// 重试执行直到成功或超时
pub async fn retry_until<F, Fut, T, E>(max_attempts: usize, delay_ms: u64, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_error: Option<E> = None;

    for _ in 0..max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                wait_ms(delay_ms).await;
            },
        }
    }

    Err(last_error.expect("At least one attempt should be made"))
}

/// 检查条件是否在超时内满足
pub async fn wait_for_condition<F, Fut>(
    timeout_ms: u64,
    check_interval_ms: u64,
    mut condition: F,
) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        wait_ms(check_interval_ms).await;
    }

    false
}

/// 生成随机字符串
pub fn random_string(len: usize) -> String {
    use rand::{distr::Alphanumeric, Rng};
    use std::iter;

    rand::rng().sample_iter(Alphanumeric).take(len).map(char::from).collect()
}

/// 生成随机数字符串
pub fn random_digits(len: usize) -> String {
    use rand::Rng;

    (0..len).map(|_| rand::rng().random_range(0..10).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_ms() {
        let start = std::time::Instant::now();
        wait_ms(100).await;
        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[test]
    fn test_random_string() {
        let s = random_string(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_random_digits() {
        let s = random_digits(10);
        assert_eq!(s.len(), 10);
        assert!(s.chars().all(|c| c.is_ascii_digit()));
    }
}
