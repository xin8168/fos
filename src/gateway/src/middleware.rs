//! # 中间件模块

use axum::http::Request;
use std::task::{Context, Poll};
use tower::Service;

/// 认证中间件
pub struct AuthMiddleware<S> {
    inner: S,
}

impl<S> AuthMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for AuthMiddleware<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // TODO: 实现认证逻辑
        self.inner.call(request)
    }
}

/// 日志中间件
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S> LoggingMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let method = request.method().clone();
        let uri = request.uri().clone();

        tracing::info!("收到请求: {} {}", method, uri);

        self.inner.call(request)
    }
}

/// 限流中间件
pub struct RateLimitMiddleware<S> {
    inner: S,
    #[allow(dead_code)]
    max_requests: usize,
}

impl<S> RateLimitMiddleware<S> {
    pub fn new(inner: S, max_requests: usize) -> Self {
        Self { inner, max_requests }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitMiddleware<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // TODO: 实现限流逻辑
        self.inner.call(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_creation() {
        // 基本创建测试
    }
}
