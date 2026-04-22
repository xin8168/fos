//! FOS Cache - 二进制入口点

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 FOS Cache v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("✅ 服务就绪");

    // 保持运行
    tokio::signal::ctrl_c().await?;
    tracing::info!("🛑 FOS Cache 已停止");

    Ok(())
}
