//! # Gateway 服务器模块

use crate::error::{GatewayError, Result};
use crate::handler::CommandHandler;
use crate::middleware::{AuthMiddleware, LoggingMiddleware, RateLimitMiddleware};
use crate::protocol::ProtocolParser;
use crate::token::TokenManager;
use crate::validator::FosValidator;
use crate::{FosCommand, SixAnchor};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// Gateway 服务器配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub port: u16,
    pub host: String,
    pub auth_enabled: bool,
    pub rate_limit_enabled: bool,
    pub request_timeout: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            auth_enabled: true,
            rate_limit_enabled: true,
            request_timeout: 30,
        }
    }
}

/// Gateway 服务器状态
#[derive(Debug)]
pub struct GatewayState {
    pub parser: ProtocolParser,
    pub handler: CommandHandler,
    pub validator: FosValidator,
    pub token_manager: TokenManager,
    pub stats: Arc<RwLock<GatewayStats>>,
}

/// Gateway 统计信息
#[derive(Debug, Default)]
pub struct GatewayStats {
    pub total_requests: u64,
    pub success_requests: u64,
    pub failed_requests: u64,
    pub blocked_requests: u64,
}

/// 命令提交请求
#[derive(Debug, Deserialize)]
pub struct SubmitCommandRequest {
    /// 命令内容（JSON格式）
    pub command: FosCommand,
    /// 是否需要令牌
    #[serde(default)]
    pub require_token: bool,
}

/// 命令提交响应
#[derive(Debug, Serialize)]
pub struct SubmitCommandResponse {
    pub success: bool,
    pub event_id: String,
    pub token: Option<String>,
    pub message: String,
}

/// 状态查询响应
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub event_id: String,
    pub status: String,
    pub created_at: i64,
}

/// Gateway 服务器
pub struct GatewayServer {
    config: GatewayConfig,
    state: Arc<GatewayState>,
}

impl GatewayServer {
    pub fn new(config: GatewayConfig) -> Self {
        let parser = ProtocolParser::new();
        let handler = CommandHandler::new();
        let validator = FosValidator::new();
        let token_manager = TokenManager::default();
        let stats = Arc::new(RwLock::new(GatewayStats::default()));

        Self {
            config,
            state: Arc::new(GatewayState { parser, handler, validator, token_manager, stats }),
        }
    }

    pub fn build_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/command", post(submit_command))
            .route("/api/v1/command/parse", post(parse_command))
            .route("/api/v1/command/validate", post(validate_command))
            .route("/api/v1/status/{id}", get(get_event_status))
            .route("/api/v1/stats", get(get_stats))
            .route("/api/v1/token/generate", post(generate_token))
            .route("/api/v1/token/validate", post(validate_token))
            .with_state(self.state.clone())
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .into_inner(),
            )
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        tracing::info!("🚀 FOS Gateway 服务启动: http://{}", addr);

        let router = self.build_router();

        axum::serve(listener, router).await.map_err(|e| GatewayError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// 健康检查
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "fos-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// 提交命令
async fn submit_command(
    State(state): State<Arc<GatewayState>>,
    Json(req): Json<SubmitCommandRequest>,
) -> Result<Json<SubmitCommandResponse>> {
    // 校验命令
    state.validator.validate_six_anchor(&req.command.anchor)?;

    // 处理命令
    let event = state.handler.handle(req.command).await?;

    // 可选生成令牌
    let token = if req.require_token {
        Some(state.token_manager.generate_execution_token(&event.id).await?)
    } else {
        None
    };

    // 更新统计
    {
        let mut stats = state.stats.write().await;
        stats.total_requests += 1;
        stats.success_requests += 1;
    }

    Ok(Json(SubmitCommandResponse {
        success: true,
        event_id: event.id.clone(),
        token,
        message: "命令已接收".to_string(),
    }))
}

/// 解析命令
async fn parse_command(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let input = payload.to_string();
    let command = state.parser.parse(&input)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "command": command
    })))
}

/// 校验命令
async fn validate_command(
    State(state): State<Arc<GatewayState>>,
    Json(anchor): Json<SixAnchor>,
) -> Result<Json<serde_json::Value>> {
    state.validator.validate_six_anchor(&anchor)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "校验通过"
    })))
}

/// 获取事件状态
async fn get_event_status(
    State(_state): State<Arc<GatewayState>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "event_id": id,
        "status": "pending",
        "message": "状态查询功能待实现"
    }))
}

/// 获取统计信息
async fn get_stats(State(state): State<Arc<GatewayState>>) -> Json<serde_json::Value> {
    let stats = state.stats.read().await;
    let token_stats = state.token_manager.get_stats().await;

    Json(serde_json::json!({
        "gateway": {
            "total_requests": stats.total_requests,
            "success_requests": stats.success_requests,
            "failed_requests": stats.failed_requests,
            "blocked_requests": stats.blocked_requests,
            "success_rate": if stats.total_requests > 0 {
                (stats.success_requests as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        },
        "tokens": {
            "total": token_stats.total,
            "active": token_stats.active_count,
            "expired": token_stats.expired_count
        }
    }))
}

/// 生成令牌
async fn generate_token(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let event_id = payload["event_id"].as_str().unwrap_or("default");

    let token = state.token_manager.generate_execution_token(event_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "token": token
    })))
}

/// 验证令牌
async fn validate_token(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let token =
        payload["token"].as_str().ok_or_else(|| GatewayError::MissingField("token".to_string()))?;

    let info = state.token_manager.validate_token(token).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "valid": true,
        "event_id": info.event_id,
        "expires_at": info.expires_at.to_rfc3339()
    })))
}
