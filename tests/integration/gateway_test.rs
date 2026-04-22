//! FOS Gateway 集成测试
//!
//! 测试 Gateway 模块与其他模块的集成

use fos_gateway::{
    FosValidator, FourElement, GatewayConfig, SixAnchor, TokenConfig, TokenManager, TokenType,
};

#[test]
fn test_validator_and_token_integration() {
    // 创建校验器
    let validator = FosValidator::new();

    // 创建有效的锚点
    let anchor = SixAnchor {
        event: "清理桌面文件".to_string(),
        steps: vec!["列出文件".to_string(), "删除临时文件".to_string()],
        judgment_logic: "文件为临时文件".to_string(),
        verification_standard: "桌面干净".to_string(),
        location: "我的电脑".to_string(),
        subject: "用户".to_string(),
    };

    // 校验应该通过
    assert!(validator.validate_six_anchor(&anchor).is_ok());
}

#[tokio::test]
async fn test_full_command_flow() {
    // 创建令牌管理器
    let token_manager = TokenManager::default();

    // 生成执行令牌
    let event_id = "test-event-001";
    let token = token_manager.generate_execution_token(event_id).await;
    assert!(token.is_ok());

    let token = token.unwrap();
    assert!(token.starts_with("fos_"));

    // 验证令牌
    let validation = token_manager.validate_token(&token).await;
    assert!(validation.is_ok());

    let info = validation.unwrap();
    assert_eq!(info.event_id, event_id);
    assert_eq!(info.token_type, TokenType::Execution);
    assert!(info.is_valid());
}

#[test]
fn test_validator_rejects_invalid_input() {
    let validator = FosValidator::new();

    // 测试空事件
    let empty_anchor = SixAnchor {
        event: String::new(),
        steps: vec!["步骤1".to_string()],
        judgment_logic: "条件".to_string(),
        verification_standard: "标准".to_string(),
        location: "位置".to_string(),
        subject: "主体".to_string(),
    };
    assert!(validator.validate_six_anchor(&empty_anchor).is_err());

    // 测试空步骤
    let empty_steps = SixAnchor {
        event: "测试事件".to_string(),
        steps: vec![],
        judgment_logic: "条件".to_string(),
        verification_standard: "标准".to_string(),
        location: "位置".to_string(),
        subject: "主体".to_string(),
    };
    assert!(validator.validate_six_anchor(&empty_steps).is_err());
}

#[test]
fn test_validator_rejects_dangerous_chars() {
    let validator = FosValidator::new();

    let dangerous_anchor = SixAnchor {
        event: "测试<script>事件".to_string(),
        steps: vec!["步骤1".to_string()],
        judgment_logic: "条件".to_string(),
        verification_standard: "标准".to_string(),
        location: "位置".to_string(),
        subject: "主体".to_string(),
    };
    assert!(validator.validate_six_anchor(&dangerous_anchor).is_err());
}

#[test]
fn test_four_element_validation() {
    let validator = FosValidator::new();

    // 有效的4要素
    let valid_element = FourElement {
        action: "列出文件".to_string(),
        target: "桌面文件".to_string(),
        condition: "桌面存在文件".to_string(),
        expected_result: "获得文件列表".to_string(),
    };
    assert!(validator.validate_four_element(&valid_element).is_ok());

    // 无效的4要素（空动作）
    let invalid_element = FourElement {
        action: String::new(),
        target: "目标".to_string(),
        condition: "条件".to_string(),
        expected_result: "结果".to_string(),
    };
    assert!(validator.validate_four_element(&invalid_element).is_err());
}

#[tokio::test]
async fn test_token_lifecycle() {
    let token_manager = TokenManager::default();

    // 生成令牌
    let token = token_manager.generate_session_token("session-001").await.unwrap();

    // 验证令牌有效
    let info = token_manager.validate_token(&token).await.unwrap();
    assert!(info.is_valid());
    assert_eq!(info.token_type, TokenType::Session);

    // 使用令牌
    let consumed = token_manager.consume_token(&token).await.unwrap();
    assert!(consumed.used); // 返回的是使用后的状态

    // 再次使用应该失败
    let result = token_manager.consume_token(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_revocation() {
    let token_manager = TokenManager::default();

    // 生成令牌
    let token = token_manager.generate_api_token("api-001").await.unwrap();

    // 撤销令牌
    token_manager.revoke_token(&token).await.unwrap();

    // 验证已撤销的令牌应该失败
    let result = token_manager.validate_token(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_statistics() {
    let token_manager = TokenManager::default();

    // 生成多个令牌
    token_manager.generate_execution_token("event-1").await.unwrap();
    token_manager.generate_session_token("event-2").await.unwrap();
    token_manager.generate_api_token("event-3").await.unwrap();

    // 获取统计
    let stats = token_manager.get_stats().await;
    assert_eq!(stats.total, 3);
    assert_eq!(stats.execution_count, 1);
    assert_eq!(stats.session_count, 1);
    assert_eq!(stats.api_count, 1);
    assert_eq!(stats.active_count, 3);
}

#[test]
fn test_gateway_config() {
    let config = GatewayConfig::default();
    assert_eq!(config.port, 8080);
    assert_eq!(config.host, "0.0.0.0");
    assert!(config.auth_enabled);
    assert!(config.rate_limit_enabled);
}

#[test]
fn test_custom_token_config() {
    let config = TokenConfig {
        execution_token_ttl_secs: 1800,
        session_token_ttl_secs: 43200,
        api_token_ttl_secs: 86400,
        max_tokens: 5000,
    };

    let _manager = TokenManager::new(config);
    // 验证管理器创建成功
}

#[tokio::test]
async fn test_multiple_tokens_same_event() {
    let token_manager = TokenManager::default();

    // 同一事件可以生成多个令牌
    let token1 = token_manager.generate_execution_token("event-001").await.unwrap();
    let token2 = token_manager.generate_execution_token("event-001").await.unwrap();

    // 两个令牌应该不同
    assert_ne!(token1, token2);

    // 两个令牌都应该有效
    assert!(token_manager.validate_token(&token1).await.is_ok());
    assert!(token_manager.validate_token(&token2).await.is_ok());
}

#[test]
fn test_validator_with_custom_config() {
    use fos_gateway::ValidatorConfig;

    let config = ValidatorConfig {
        max_event_length: 50,
        max_steps_count: 5,
        max_step_length: 100,
        max_judgment_length: 100,
        max_verification_length: 100,
        max_location_length: 50,
        max_subject_length: 30,
    };

    let validator = FosValidator::with_config(config);

    // 创建一个长事件名
    let long_event_anchor = SixAnchor {
        event: "这是一个非常长的事件名称超过五十个字符的限制应该被拒绝".to_string(),
        steps: vec!["步骤1".to_string()],
        judgment_logic: "条件".to_string(),
        verification_standard: "标准".to_string(),
        location: "位置".to_string(),
        subject: "主体".to_string(),
    };

    // 应该因为长度超过限制而失败
    assert!(validator.validate_six_anchor(&long_event_anchor).is_err());
}
