# FOS Dockerfile
# 多阶段构建，最小化镜像大小
# 使用 cargo-chef 优化构建缓存

# ==================== 构建阶段 ====================
FROM rust:1.76-slim AS chef

# 安装 cargo-chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# 准备依赖缓存
FROM chef AS planner
COPY src/ ./
RUN cargo chef prepare --recipe-path recipe.json

# 构建依赖
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 构建依赖（缓存层）
RUN cargo chef cook --release --recipe-path recipe.json

# 复制源代码并构建
COPY src/ ./
RUN cargo build --release --workspace

# ==================== 运行阶段 ====================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/fos-gateway /usr/local/bin/
COPY --from=builder /app/target/release/fos-validator /usr/local/bin/
COPY --from=builder /app/target/release/fos-bus /usr/local/bin/
COPY --from=builder /app/target/release/fos-memory /usr/local/bin/
COPY --from=builder /app/target/release/fos-audit /usr/local/bin/
COPY --from=builder /app/target/release/fos-sandbox /usr/local/bin/
COPY --from=builder /app/target/release/fos-skills /usr/local/bin/
COPY --from=builder /app/target/release/fos-mcp /usr/local/bin/
COPY --from=builder /app/target/release/fos-rollback /usr/local/bin/
COPY --from=builder /app/target/release/fos-permission /usr/local/bin/
COPY --from=builder /app/target/release/fos-eventlog /usr/local/bin/
COPY --from=builder /app/target/release/fos-notifier /usr/local/bin/
COPY --from=builder /app/target/release/fos-bootstrap /usr/local/bin/
COPY --from=builder /app/target/release/fos-shutdown /usr/local/bin/
COPY --from=builder /app/target/release/fos-health /usr/local/bin/
COPY --from=builder /app/target/release/fos-transaction /usr/local/bin/
COPY --from=builder /app/target/release/fos-lock /usr/local/bin/
COPY --from=builder /app/target/release/fos-idempotency /usr/local/bin/
COPY --from=builder /app/target/release/fos-migration /usr/local/bin/
COPY --from=builder /app/target/release/fos-backup /usr/local/bin/
COPY --from=builder /app/target/release/fos-ratelimiter /usr/local/bin/
COPY --from=builder /app/target/release/fos-plugin /usr/local/bin/
COPY --from=builder /app/target/release/fos-schedule /usr/local/bin/
COPY --from=builder /app/target/release/fos-cache /usr/local/bin/
COPY --from=builder /app/target/release/fos-config /usr/local/bin/
COPY --from=builder /app/target/release/fos-monitoring /usr/local/bin/

# 创建非root用户
RUN useradd -r -s /bin/false fos

# 设置权限
RUN chown -R fos:fos /app

# 切换到非root用户
USER fos

# 暴露端口
EXPOSE 8080 8081 8082 9090

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 默认入口
ENTRYPOINT ["fos-gateway"]
CMD ["--config", "/app/config/default.toml"]
