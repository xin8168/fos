# FOS 开发指南

## 环境准备

### 必需软件

| 软件 | 版本 | 用途 |
|-----|------|------|
| Rust | ≥1.76 | 核心开发 |
| Python | ≥3.11 | CLI开发 |
| PostgreSQL | ≥15 | 数据库 |
| Redis | ≥7.0 | 缓存 |
| Docker | ≥24.0 | 容器化 |

### 安装Rust

```bash
# 安装rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加目标平台
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu

# 安装组件
rustup component add rustfmt clippy rust-analyzer rust-src
```

### 克隆项目

```bash
git clone https://github.com/fos-platform/fos.git
cd fos
```

---

## 项目结构

```
fos/
├── src/                    # 源代码
│   ├── gateway/           # 协议网关
│   ├── validator/         # 规则校验
│   ├── bus/               # 执行总线
│   ├── memory/            # 硬记忆库
│   ├── audit/             # 拦截日志
│   └── ...                # 其他模块
├── tests/                  # 测试
│   ├── integration/       # 集成测试
│   ├── test-utils/        # 测试工具
│   └── e2e/               # 端到端测试
├── docs/                   # 文档
├── scripts/                # 脚本
└── .github/                # CI/CD
```

---

## 开发流程

### 1. 创建功能分支

```bash
git checkout -b feature/your-feature
```

### 2. 编写代码

遵循TDD流程：
1. 编写失败测试 (Red)
2. 实现最小代码 (Green)
3. 优化重构 (Refactor)

### 3. 运行测试

```bash
# 运行所有测试
cargo test --workspace --all-targets

# 运行特定模块测试
cargo test -p fos-gateway

# 运行覆盖率
cargo tarpaulin --workspace
```

### 4. 代码检查

```bash
# 格式检查
cargo fmt --all -- --check

# 静态检查
cargo clippy --workspace --all-targets -- -D warnings

# 安全审计
cargo audit

# 依赖检查
cargo deny check
```

### 5. 提交代码

```bash
# 添加更改
git add .

# 提交（遵循Conventional Commits）
git commit -m "feat(gateway): add new validation logic"

# 推送
git push origin feature/your-feature
```

---

## 代码规范

### Rust代码风格

```rust
//! 模块文档注释

/// 公共项文档注释
pub struct Example {
    /// 字段文档注释
    pub field: String,
}

// 错误处理使用thiserror
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("错误描述: {0}")]
    ExampleError(String),
}

// 异步使用tokio
pub async fn example() -> Result<()> {
    Ok(())
}
```

### 提交信息规范

```
<type>(<scope>): <description>

type: feat, fix, docs, style, refactor, test, chore
scope: gateway, validator, bus, memory, audit, etc.
```

示例：
```
feat(gateway): add token validation
fix(validator): correct permission check logic
docs(architecture): update system diagram
```

---

## 模块开发

### 创建新模块

1. 在 `src/` 下创建模块目录
2. 创建 `Cargo.toml`
3. 创建 `src/lib.rs`, `src/error.rs`, `src/config.rs`
4. 更新工作空间 `Cargo.toml`

### 模块模板

```rust
// src/xxx/src/lib.rs
//! # FOS XXX 模块

pub mod error;
pub mod config;

pub use error::{Error, Result};
pub use config::Config;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

---

## 调试

### 日志

```bash
# 设置日志级别
RUST_LOG=debug cargo run

# 查看特定模块日志
RUST_LOG=fos_gateway=debug cargo run
```

### 测试调试

```bash
# 运行单个测试
cargo test test_name -- --nocapture

# 显示测试输出
cargo test -- --nocapture --test-threads=1
```

---

## 发布流程

1. 更新版本号
2. 运行完整测试套件
3. 创建Git标签
4. CI/CD自动构建发布

---

**版本**: v1.0.0  
**最后更新**: 2026-03-09
