# FOS - 确定性执行板

> **零风险、零回退、零混乱** 的设备控制平台

## 概述

FOS（确定性执行板）是一个工业级软件项目，通过确定性的、基于规则的执行来确保设备控制的安全性。

### 核心特性

- **确定性执行** - 基于规则的执行引擎，确保每次操作都可预测
- **事务管理** - 分布式事务协调，保证数据一致性
- **分布式锁** - 高并发场景下的资源保护
- **幂等控制** - 防止重复操作导致的数据不一致
- **回滚引擎** - 支持多级回滚和快照恢复
- **缓存系统** - 高性能本地和分布式缓存
- **插件系统** - 可扩展的插件架构
- **调度系统** - 定时和延迟任务调度
- **混沌工程** - 内置故障注入和恢复测试

## 快速开始

### 前置条件

- Rust 1.76+
- Cargo
- Docker (可选)
- Kubernetes (可选)

### 本地开发

```bash
# 克隆项目
git clone https://github.com/fos-platform/fos.git
cd fos

# 构建所有模块
cd src
cargo build --release --workspace

# 运行测试
cargo test --workspace

# 运行基准测试
cd benches
cargo bench --bench system_benchmark
```

### Docker 部署

```bash
# 构建镜像
docker build -t fos-platform/fos:latest .

# 运行容器
docker run -d \
  --name fos-gateway \
  -p 8080:8080 \
  -p 9090:9090 \
  fos-platform/fos:latest
```

### Kubernetes 部署

```bash
# 应用配置
kubectl apply -f deploy/k8s/deployment.yaml

# 验证部署
kubectl get pods -n fos-system
kubectl get services -n fos-system
```

## 架构

### 模块结构

```
src/
├── gateway/          # API 网关
├── validator/        # 规则验证引擎
├── memory/           # 事件存储
├── bus/              # 消息总线
├── audit/            # 审计日志
├── sandbox/          # 沙箱隔离
├── skills/           # 技能模块
├── mcp/              # 设备控制协议
├── rollback/         # 回滚引擎
├── permission/       # 权限管理
├── eventlog/         # 事件日志
├── notifier/         # 通知系统
├── bootstrap/        # 启动引导
├── shutdown/         # 优雅关闭
├── health/           # 健康检查
├── transaction/      # 事务管理
├── lock/             # 分布式锁
├── idempotency/      # 幂等控制
├── migration/        # 数据迁移
├── backup/           # 数据备份
├── ratelimiter/      # 速率限制
├── plugin/           # 插件系统
├── schedule/         # 调度系统
├── cache/            # 缓存系统
├── config/           # 配置管理
└── monitoring/       # 监控模块
```

### 扩展模块

```
src/
├── stability/        # 稳定性测试
├── security/         # 安全测试
├── chaos/            # 混沌工程
└── benches/          # 性能基准测试
```

## API 文档

### 健康检查

```bash
GET /health
```

响应:
```json
{
  "status": "healthy",
  "timestamp": "2026-03-19T10:00:00Z"
}
```

### 事务管理

```bash
POST /api/transactions
Content-Type: application/json

{
  "name": "device_control",
  "participants": ["service1", "service2"]
}
```

### 分布式锁

```bash
POST /api/locks/acquire
Content-Type: application/json

{
  "key": "resource_key",
  "owner": "service_1",
  "timeout_secs": 30
}
```

## 配置

### 环境变量

| 变量 | 描述 | 默认值 |
|-----|------|-------|
| `RUST_LOG` | 日志级别 | `info` |
| `FOS_CONFIG_PATH` | 配置文件路径 | `/app/config/default.toml` |
| `DATABASE_URL` | 数据库连接字符串 | - |
| `REDIS_URL` | Redis 连接字符串 | - |

### 配置文件

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://fos:fos@localhost:5432/fos"
max_connections = 10

[logging]
level = "info"
format = "json"

[cache]
max_size = 10000
ttl_seconds = 3600

[lock]
timeout_seconds = 30
wait_timeout_seconds = 10
```

## 监控

### Prometheus 指标

- `http_requests_total` - HTTP 请求总数
- `http_request_duration_seconds` - 请求延迟
- `fos_transaction_total` - 事务总数
- `fos_transaction_failures_total` - 事务失败数
- `fos_lock_wait_timeouts_total` - 锁等待超时数
- `fos_cache_hits_total` - 缓存命中数
- `fos_cache_misses_total` - 缓存未命中数

### Grafana 面板

导入 `deploy/monitoring/grafana_dashboard.json` 到 Grafana 以创建完整的监控面板。

## 测试

### 单元测试

```bash
cd src
cargo test --workspace
```

### 集成测试

```bash
cd tests
cargo test
```

### 性能基准测试

```bash
cd src/benches
cargo bench --bench system_benchmark
```

### 稳定性测试

```bash
cd src/stability
cargo test
```

### 安全测试

```bash
cd src/security
cargo test
```

### 混沌工程测试

```bash
cd src/chaos
cargo test
```

## 部署

### 生产环境要求

- Kubernetes 1.24+
- PostgreSQL 14+
- Redis 7+
- Prometheus + Grafana
- ELK Stack (日志收集)

### 部署步骤

1. 配置 Kubernetes 集群
2. 应用 `deploy/k8s/deployment.yaml`
3. 配置 Prometheus 监控
4. 配置 Grafana 仪表板
5. 配置告警规则
6. 验证健康检查

## 安全

### 安全最佳实践

- 所有模块使用 `unsafe_code = "forbid"`
- 严格的编译警告处理 (`warnings = "deny"`)
- 完整的错误处理和日志记录
- 非 root 用户运行容器
- 最小权限原则

### 渗透测试

```bash
cd src/security
cargo test
```

## 故障排除

### 常见问题

**Q: 编译失败？**
A: 确保使用 Rust 1.76+ 并运行 `cargo update`

**Q: 测试超时？**
A: 增加超时时间或检查网络连接

**Q: Pod 无法启动？**
A: 检查 `kubectl describe pod <pod-name>` 获取详细信息

### 日志查询

```bash
# 查看 Gateway 日志
kubectl logs -n fos-system -l component=gateway -f

# 查看特定 Pod 日志
kubectl logs -n fos-system <pod-name> -f
```

## 贡献

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

Apache-2.0

## 联系方式

- 项目主页: https://github.com/fos-platform/fos
- 问题反馈: https://github.com/fos-platform/fos/issues
