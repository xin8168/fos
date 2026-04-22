//! FOS Gateway - 主程序入口
//!
//! FOS 协议网关服务入口点

use fos_gateway::server::GatewayServer;
use fos_gateway::config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 FOS Gateway 启动中...");

    // 加载配置
    let config = Config::from_env().unwrap_or_default();
    tracing::info!("📋 配置加载完成: {}", config.server.addr);

    // 创建并启动服务器
    let server = GatewayServer::new(fos_gateway::server::GatewayConfig {
        port: config.server.addr.port(),
        host: config.server.addr.ip().to_string(),
        ..Default::default()
    });

    // 启动服务
    server.start().await?;

    tracing::info!("✅ FOS Gateway 已停止");

    Ok(())
}
