//! # Cron表达式解析器
//!
//! 解析和验证标准的Cron表达式

use std::collections::HashSet;

/// Cron表达式秒数（0-59）
pub type CronSecond = u8;
/// Cron表达式分钟（0-59）
pub type CronMinute = u8;
/// Cron表达式小时（0-23）
pub type CronHour = u8;
/// Cron表达式日期（1-31）
pub type CronDay = u8;
/// Cron表达式月份（1-12）
pub type CronMonth = u8;
/// Cron表达式星期（0-6，0=周日）
pub type CronDayOfWeek = u8;

/// Cron表达式
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronExpression {
    /// 秒数
    pub seconds: HashSet<CronSecond>,
    /// 分钟
    pub minutes: HashSet<CronMinute>,
    /// 小时
    pub hours: HashSet<CronHour>,
    /// 日期
    pub days: HashSet<CronDay>,
    /// 月份
    pub months: HashSet<CronMonth>,
    /// 星期
    pub days_of_week: HashSet<CronDayOfWeek>,
}

impl Default for CronExpression {
    fn default() -> Self {
        Self {
            seconds: vec![0].into_iter().collect(),
            minutes: vec![0].into_iter().collect(),
            hours: vec![0].into_iter().collect(),
            days: vec![1].into_iter().collect(),
            months: (1..13).collect(),
            days_of_week: vec![1].into_iter().collect(),
        }
    }
}

impl CronExpression {
    /// 解析Cron表达式
    pub fn parse(cron_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = cron_str.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(format!("Invalid cron expression: expected 6 parts, got {}", parts.len()));
        }

        Ok(Self {
            seconds: Self::parse_field(parts[0], 0, 59)?,
            minutes: Self::parse_field(parts[1], 0, 59)?,
            hours: Self::parse_field(parts[2], 0, 23)?,
            days: Self::parse_field(parts[3], 1, 31)?,
            months: Self::parse_field(parts[4], 1, 12)?,
            days_of_week: Self::parse_field(parts[5], 0, 6)?,
        })
    }

    /// 解析单个字段
    fn parse_field(field: &str, min: u8, max: u8) -> Result<HashSet<u8>, String> {
        let mut values = HashSet::new();

        // 处理通配符 *
        if field == "*" {
            for i in min..=max {
                values.insert(i);
            }
            return Ok(values);
        }

        // 处理列表 1,2,3
        for part in field.split(',') {
            // 处理范围 1-5
            if part.contains('-') {
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() != 2 {
                    return Err(format!("Invalid range: {}", part));
                }

                let start: u8 = range_parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid range start: {}", range_parts[0]))?;
                let end_and_step: Vec<&str> = range_parts[1].split('/').collect();
                let end: u8 = end_and_step[0]
                    .parse()
                    .map_err(|_| format!("Invalid range end: {}", end_and_step[0]))?;

                // 处理范围步长 1-5/2
                if end_and_step.len() > 1 {
                    let step: u8 = end_and_step[1]
                        .parse()
                        .map_err(|_| format!("Invalid step: {}", end_and_step[1]))?;
                    for i in (start..=end).step_by(step as usize) {
                        values.insert(i);
                    }
                } else {
                    for i in start..=end {
                        values.insert(i);
                    }
                }
            } else if part.contains('/') {
                // 处理步长 */5 或 0-10/2
                let step_parts: Vec<&str> = part.split('/').collect();
                if step_parts.len() != 2 {
                    return Err(format!("Invalid step: {}", part));
                }

                let step: u8 = step_parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid step: {}", step_parts[1]))?;

                let range_part = step_parts[0];
                if range_part == "*" {
                    for i in (min..=max).step_by(step as usize) {
                        values.insert(i);
                    }
                } else {
                    // 处理范围步长
                    if range_part.contains('-') {
                        let range_parts: Vec<&str> = range_part.split('-').collect();
                        let start: u8 = range_parts[0]
                            .parse()
                            .map_err(|_| format!("Invalid range: {}", range_part))?;
                        let end: u8 = range_parts[1]
                            .parse()
                            .map_err(|_| format!("Invalid range: {}", range_part))?;

                        for i in (start..=end).step_by(step as usize) {
                            values.insert(i);
                        }
                    } else {
                        // 处理单个值步长
                        let value: u8 = range_part
                            .parse()
                            .map_err(|_| format!("Invalid value: {}", range_part))?;
                        for i in (value..=max).step_by(step as usize) {
                            values.insert(i);
                        }
                    }
                }
            } else {
                // 处理单个值
                let value: u8 =
                    part.trim().parse().map_err(|_| format!("Invalid value: {}", part))?;

                if value < min || value > max {
                    return Err(format!("Value {} out of range [{}, {}]", value, min, max));
                }

                values.insert(value);
            }
        }

        if values.is_empty() {
            return Err("No valid values found".to_string());
        }

        Ok(values)
    }

    /// 检查给定时间是否匹配Cron表达式
    pub fn matches(
        &self,
        second: u8,
        minute: u8,
        hour: u8,
        day: u8,
        month: u8,
        day_of_week: u8,
    ) -> bool {
        self.seconds.contains(&second)
            && self.minutes.contains(&minute)
            && self.hours.contains(&hour)
            && self.days.contains(&day)
            && self.months.contains(&month)
            && self.days_of_week.contains(&day_of_week)
    }

    /// 获取下一次执行时间
    pub fn next_run(
        &self,
        _current_second: u8,
        _current_minute: u8,
        _current_hour: u8,
    ) -> Option<(u8, u8, u8)> {
        // 简化版本：返回第一个有效时间
        let sec = self.seconds.iter().next().copied()?;
        let min = self.minutes.iter().next().copied()?;
        let hour = self.hours.iter().next().copied()?;
        Some((sec, min, hour))
    }

    /// 验证Cron表达式是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.seconds.is_empty() {
            return Err("Seconds field is empty".to_string());
        }
        if self.minutes.is_empty() {
            return Err("Minutes field is empty".to_string());
        }
        if self.hours.is_empty() {
            return Err("Hours field is empty".to_string());
        }
        if self.days.is_empty() {
            return Err("Days field is empty".to_string());
        }
        if self.months.is_empty() {
            return Err("Months field is empty".to_string());
        }
        if self.days_of_week.is_empty() {
            return Err("Days of week field is empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wildcard() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();
        assert_eq!(cron.seconds.len(), 1);
        assert!(cron.seconds.contains(&0));
        assert_eq!(cron.minutes.len(), 60);
    }

    #[test]
    fn test_parse_single_values() {
        let cron = CronExpression::parse("0 30 12 * * *").unwrap();
        assert!(cron.seconds.contains(&0));
        assert!(cron.minutes.contains(&30));
        assert!(cron.hours.contains(&12));
    }

    #[test]
    fn test_parse_list() {
        let cron = CronExpression::parse("0 0,15,30,45 * * * *").unwrap();
        assert!(cron.minutes.contains(&0));
        assert!(cron.minutes.contains(&15));
        assert!(cron.minutes.contains(&30));
        assert!(cron.minutes.contains(&45));
    }

    #[test]
    fn test_parse_range() {
        let cron = CronExpression::parse("0 10-20 * * * *").unwrap();
        for i in 10..=20 {
            assert!(cron.minutes.contains(&i));
        }
    }

    #[test]
    fn test_parse_step() {
        let cron = CronExpression::parse("0 */10 * * * *").unwrap();
        assert!(cron.minutes.contains(&0));
        assert!(cron.minutes.contains(&10));
        assert!(cron.minutes.contains(&20));
        assert!(cron.minutes.contains(&30));
    }

    #[test]
    fn test_matches_exact() {
        let cron = CronExpression::parse("0 30 12 1 1 1").unwrap();
        assert!(cron.matches(0, 30, 12, 1, 1, 1));
        assert!(!cron.matches(1, 30, 12, 1, 1, 1));
    }

    #[test]
    fn test_matches_wildcard() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();
        assert!(cron.matches(0, 30, 12, 1, 1, 1));
    }

    #[test]
    fn test_validate_valid() {
        let cron = CronExpression::parse("0 30 12 * * *").unwrap();
        assert!(cron.validate().is_ok());
    }
}
