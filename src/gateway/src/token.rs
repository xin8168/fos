//! FOS 令牌管理模块
//!
//! 实现安全的令牌生成和验证

use crate::error::{GatewayError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 令牌类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    /// 执行令牌
    Execution,
    /// 会话令牌
    Session,
    /// API令牌
    Api,
}

/// 令牌信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// 令牌ID
    pub token: String,
    /// 令牌类型
    pub token_type: TokenType,
    /// 关联的事件ID
    pub event_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 是否已使用
    pub used: bool,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl TokenInfo {
    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查令牌是否有效
    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }
}

/// 令牌配置
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// 执行令牌有效期（秒）
    pub execution_token_ttl_secs: i64,
    /// 会话令牌有效期（秒）
    pub session_token_ttl_secs: i64,
    /// API令牌有效期（秒）
    pub api_token_ttl_secs: i64,
    /// 最大令牌数
    pub max_tokens: usize,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            execution_token_ttl_secs: 3600, // 1小时
            session_token_ttl_secs: 86400,  // 24小时
            api_token_ttl_secs: 2592000,    // 30天
            max_tokens: 10000,
        }
    }
}

/// 令牌管理器
#[derive(Debug)]
pub struct TokenManager {
    config: TokenConfig,
    tokens: Arc<RwLock<HashMap<String, TokenInfo>>>,
}

impl TokenManager {
    /// 创建新令牌管理器
    pub fn new(config: TokenConfig) -> Self {
        Self { config, tokens: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 生成执行令牌
    pub async fn generate_execution_token(&self, event_id: &str) -> Result<String> {
        self.generate_token(event_id, TokenType::Execution, self.config.execution_token_ttl_secs)
            .await
    }

    /// 生成会话令牌
    pub async fn generate_session_token(&self, event_id: &str) -> Result<String> {
        self.generate_token(event_id, TokenType::Session, self.config.session_token_ttl_secs).await
    }

    /// 生成API令牌
    pub async fn generate_api_token(&self, event_id: &str) -> Result<String> {
        self.generate_token(event_id, TokenType::Api, self.config.api_token_ttl_secs).await
    }

    /// 生成令牌
    async fn generate_token(
        &self,
        event_id: &str,
        token_type: TokenType,
        ttl_secs: i64,
    ) -> Result<String> {
        let mut tokens = self.tokens.write().await;

        // 检查令牌数量限制
        if tokens.len() >= self.config.max_tokens {
            // 清理过期令牌
            self.cleanup_expired_tokens_locked(&mut tokens);

            if tokens.len() >= self.config.max_tokens {
                return Err(GatewayError::Internal("令牌数量已达上限".to_string()));
            }
        }

        let now = Utc::now();
        let token = format!("fos_{}", Uuid::new_v4());

        let token_info = TokenInfo {
            token: token.clone(),
            token_type,
            event_id: event_id.to_string(),
            created_at: now,
            expires_at: now + Duration::seconds(ttl_secs),
            used: false,
            metadata: HashMap::new(),
        };

        tokens.insert(token.clone(), token_info);
        Ok(token)
    }

    /// 验证令牌
    pub async fn validate_token(&self, token: &str) -> Result<TokenInfo> {
        let tokens = self.tokens.read().await;

        let info = tokens
            .get(token)
            .ok_or_else(|| GatewayError::Unauthorized("无效的令牌".to_string()))?;

        if info.is_expired() {
            return Err(GatewayError::Unauthorized("令牌已过期".to_string()));
        }

        if info.used {
            return Err(GatewayError::Unauthorized("令牌已使用".to_string()));
        }

        Ok(info.clone())
    }

    /// 使用令牌（标记为已使用）
    pub async fn consume_token(&self, token: &str) -> Result<TokenInfo> {
        let mut tokens = self.tokens.write().await;

        let info = tokens
            .get_mut(token)
            .ok_or_else(|| GatewayError::Unauthorized("无效的令牌".to_string()))?;

        if info.is_expired() {
            return Err(GatewayError::Unauthorized("令牌已过期".to_string()));
        }

        if info.used {
            return Err(GatewayError::Unauthorized("令牌已使用".to_string()));
        }

        info.used = true;
        Ok(info.clone())
    }

    /// 撤销令牌
    pub async fn revoke_token(&self, token: &str) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        tokens
            .remove(token)
            .map(|_| ())
            .ok_or_else(|| GatewayError::Unauthorized("无效的令牌".to_string()))
    }

    /// 清理过期令牌
    pub async fn cleanup_expired_tokens(&self) -> usize {
        let mut tokens = self.tokens.write().await;
        self.cleanup_expired_tokens_locked(&mut tokens)
    }

    fn cleanup_expired_tokens_locked(&self, tokens: &mut HashMap<String, TokenInfo>) -> usize {
        let now = Utc::now();
        let expired: Vec<String> = tokens
            .iter()
            .filter(|(_, info)| info.expires_at < now)
            .map(|(token, _)| token.clone())
            .collect();

        let count = expired.len();
        for token in expired {
            tokens.remove(&token);
        }
        count
    }

    /// 获取令牌统计
    pub async fn get_stats(&self) -> TokenStats {
        let tokens = self.tokens.read().await;
        let now = Utc::now();

        let mut stats = TokenStats::default();
        stats.total = tokens.len();

        for info in tokens.values() {
            match info.token_type {
                TokenType::Execution => stats.execution_count += 1,
                TokenType::Session => stats.session_count += 1,
                TokenType::Api => stats.api_count += 1,
            }

            if info.is_expired() {
                stats.expired_count += 1;
            }
            if info.used {
                stats.used_count += 1;
            }
        }

        stats.active_count = stats.total - stats.expired_count - stats.used_count;
        stats
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new(TokenConfig::default())
    }
}

/// 令牌统计
#[derive(Debug, Default)]
pub struct TokenStats {
    pub total: usize,
    pub active_count: usize,
    pub expired_count: usize,
    pub used_count: usize,
    pub execution_count: usize,
    pub session_count: usize,
    pub api_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_execution_token() {
        let manager = TokenManager::default();
        let token = manager.generate_execution_token("event-001").await;
        assert!(token.is_ok());
        assert!(token.unwrap().starts_with("fos_"));
    }

    #[tokio::test]
    async fn test_validate_token() {
        let manager = TokenManager::default();
        let token = manager.generate_execution_token("event-001").await.unwrap();

        let result = manager.validate_token(&token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_consume_token() {
        let manager = TokenManager::default();
        let token = manager.generate_execution_token("event-001").await.unwrap();

        let result = manager.consume_token(&token).await;
        assert!(result.is_ok());

        // 再次使用应该失败
        let result2 = manager.consume_token(&token).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_revoke_token() {
        let manager = TokenManager::default();
        let token = manager.generate_execution_token("event-001").await.unwrap();

        let result = manager.revoke_token(&token).await;
        assert!(result.is_ok());

        // 验证已撤销的令牌应该失败
        let result2 = manager.validate_token(&token).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_token_stats() {
        let manager = TokenManager::default();

        manager.generate_execution_token("event-001").await.unwrap();
        manager.generate_session_token("event-002").await.unwrap();
        manager.generate_api_token("event-003").await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 3);
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.session_count, 1);
        assert_eq!(stats.api_count, 1);
    }
}
