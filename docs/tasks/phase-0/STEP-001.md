# FOS-STEP-001: 项目结构初始化

**版本**: v1.0.0  
**创建日期**: 2026-03-09  
**状态**: 待执行  
**所属阶段**: Phase 0 - 基础设施搭建  

---

## 前情提要

### 上下文背景
- 项目刚刚启动，需要建立完整的开发基础设施
- 已有部分代码框架，但需要按照工业级标准重新组织
- 必须遵循FOS九大铁律和强制约束

### 依赖关系
- 无前置依赖（本项目第一步）
- 后续所有步骤依赖本步骤完成

### 现有资产
```
已有模块框架：
├── gateway/     - 协议网关（需完善）
├── validator/   - 规则校验（需完善）
├── bus/         - 执行总线（需完善）
├── memory/      - 硬记忆库（需完善）
├── sandbox/     - 沙箱隔离（需完善）
├── skills/      - 执行单元（需完善）
├── mcp/         - 设备管控（需完善）
├── config/      - 配置管理（需完善）
├── monitoring/  - 监控服务（需完善）
└── cli/         - 命令行（需完善）
```

---

## 当下任务目标

### 主要目标
建立符合工业级标准的Rust项目结构，包括：
1. 规范的项目目录布局
2. Cargo工作空间配置
3. 模块依赖管理
4. 代码风格配置
5. 文档结构初始化

### 预期交付物
```
交付物清单：
├── 完整的项目目录结构
├── Cargo.toml工作空间配置（更新）
├── .rustfmt.toml代码风格配置
├── clippy.toml静态检查配置
├── .gitignore版本控制配置
├── docs/文档目录结构
├── tests/测试目录结构
├── benches/性能测试目录
├── examples/示例目录
└── scripts/脚本目录
```

### 完成标准
- [ ] 目录结构符合Rust社区最佳实践
- [ ] 所有模块可独立编译
- [ ] 代码风格配置生效
- [ ] 静态检查配置生效
- [ ] 文档结构完整

---

## 技术栈约束

### 语言版本
- Rust: ≥1.75 (推荐1.76+)
- Edition: 2021

### Cargo配置约束
```toml
# Cargo.toml 必须包含的配置

[workspace]
resolver = "2"
members = [...]  # 所有模块

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/fos-platform/fos"
authors = ["FOS Team"]

[workspace.dependencies]
# 统一依赖版本管理

[workspace.lint.rust]
warnings = "deny"
unsafe_code = "forbid"
missing_docs = "deny"

[workspace.lint.clippy]
all = "deny"
pedantic = "deny"
```

### 代码风格约束
```rust
// 必须遵守的代码风格

// 1. 所有公共项必须有文档注释
/// 模块描述
pub struct ExampleStruct {
    /// 字段描述
    pub field: String,
}

// 2. 错误类型必须使用thiserror
#[derive(Debug, thiserror::Error)]
pub enum ExampleError {
    #[error("错误描述")]
    ErrorVariant,
}

// 3. 异步函数必须使用tokio
pub async fn example_async() -> Result<()> {
    Ok(())
}

// 4. 禁止unwrap/expect（除非测试代码）
// 禁止: let x = option.unwrap();
// 允许: let x = option.ok_or(Error::NotFound)?;

// 5. 所有常量必须有文档和类型
/// 超时时间（秒）
const TIMEOUT_SECS: u64 = 30;
```

### 目录结构约束
```
项目根目录/
├── Cargo.toml           # 工作空间配置
├── Cargo.lock           # 依赖锁定
├── .rustfmt.toml        # 格式化配置
├── clippy.toml          # Clippy配置
├── .gitignore           # Git忽略
├── LICENSE              # 许可证
├── README.md            # 项目说明
├── src/                 # 源代码
│   ├── gateway/         # 协议网关
│   ├── validator/       # 规则校验
│   ├── bus/             # 执行总线
│   ├── memory/          # 硬记忆库
│   ├── audit/           # 拦截日志（新增）
│   ├── sandbox/         # 沙箱隔离
│   ├── skills/          # 执行单元
│   ├── mcp/             # 设备管控
│   ├── rollback/        # 回滚引擎（新增）
│   ├── permission/      # 权限中心（新增）
│   ├── eventlog/        # 事件日志（新增）
│   ├── notifier/        # 通知服务（新增）
│   ├── bootstrap/       # 启动引导（新增）
│   ├── shutdown/        # 优雅关闭（新增）
│   ├── health/          # 健康检查（新增）
│   ├── transaction/     # 事务管理（新增）
│   ├── lock/            # 分布式锁（新增）
│   ├── idempotency/     # 幂等控制（新增）
│   ├── migration/       # 数据迁移（新增）
│   ├── backup/          # 数据备份（新增）
│   ├── ratelimiter/     # 限流控制（新增）
│   ├── plugin/          # 插件系统（新增）
│   ├── schedule/        # 定时调度（新增）
│   ├── cache/           # 多级缓存（新增）
│   ├── config/          # 配置管理
│   ├── monitoring/      # 监控服务
│   └── cli/             # 命令行
├── tests/               # 集成测试
│   ├── integration/     # 集成测试
│   ├── e2e/             # 端到端测试
│   └── common/          # 测试公共代码
├── benches/             # 性能测试
├── examples/            # 示例代码
├── docs/                # 文档
│   ├── tasks/           # 任务文档
│   ├── reports/         # 完成报告
│   ├── architecture/    # 架构文档
│   └── api/             # API文档
└── scripts/             # 脚本
    ├── build/           # 构建脚本
    ├── test/            # 测试脚本
    └── deploy/          # 部署脚本
```

---

## 验收标准

### 功能验收标准
| 序号 | 标准 | 验证方法 |
|-----|------|---------|
| F1 | 所有模块目录存在 | 目录检查 |
| F2 | 所有Cargo.toml有效 | cargo check |
| F3 | 工作空间成员完整 | cargo metadata |
| F4 | 新增模块骨架代码存在 | 文件检查 |

### 性能验收标准
| 序号 | 标准 | 验证方法 |
|-----|------|---------|
| P1 | cargo check < 30秒 | 时间测量 |
| P2 | cargo build --release < 5分钟 | 时间测量 |
| P3 | 依赖编译缓存有效 | 重复编译测试 |

### 安全验收标准
| 序号 | 标准 | 验证方法 |
|-----|------|---------|
| S1 | 无unsafe代码 | cargo geiger |
| S2 | 无已知漏洞依赖 | cargo audit |
| S3 | 许可证兼容 | cargo deny |

### 代码质量验收标准
| 序号 | 标准 | 验证方法 |
|-----|------|---------|
| Q1 | rustfmt检查通过 | cargo fmt --check |
| Q2 | clippy检查通过 | cargo clippy |
| Q3 | 无警告 | cargo check |
| Q4 | 文档生成成功 | cargo doc |

---

## 测试方法

### 单元测试要求
本步骤为结构初始化，不需要单元测试，但需要验证：

```bash
# 1. 验证Cargo工作空间
cargo metadata --format-version=1 | jq '.workspace_members'

# 2. 验证所有模块可编译
cargo check --workspace

# 3. 验证代码风格
cargo fmt --check

# 4. 验证静态检查
cargo clippy --workspace --all-targets

# 5. 验证文档生成
cargo doc --workspace --no-deps
```

### 集成测试要求
创建基础集成测试框架：

```rust
// tests/integration/main.rs
//! FOS 集成测试入口

mod common;

#[test]
fn test_workspace_structure() {
    // 验证工作空间结构
    assert!(std::path::Path::new("src/gateway").exists());
    assert!(std::path::Path::new("src/validator").exists());
    // ... 验证所有模块
}
```

### 测试覆盖率要求
本步骤为结构初始化，测试覆盖率要求：
- 配置文件正确性：100%
- 目录结构完整性：100%

---

## 接口协议标准

### 模块公共接口规范

每个模块必须导出的标准接口：

```rust
// src/xxx/src/lib.rs

//! # 模块名称
//! 
//! 模块描述
//!
//! ## 核心职责
//! - 职责1
//! - 职责2
//!
//! ## 安全铁律
//! - 铁律1
//! - 铁律2

pub mod error;
pub mod config;
// ... 其他子模块

// 标准导出
pub use error::{Error, Result};
pub use config::Config;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");
```

### 错误类型标准

```rust
// src/xxx/src/error.rs

//! 错误类型定义

use thiserror::Error;

/// 模块错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),
    
    /// 操作错误
    #[error("操作错误: {0}")]
    Operation(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 模块Result类型
pub type Result<T> = std::result::Result<T, Error>;
```

### 配置类型标准

```rust
// src/xxx/src/config.rs

//! 配置类型定义

use serde::{Deserialize, Serialize};

/// 模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 配置项1
    pub item1: String,
    
    /// 配置项2
    pub item2: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            item1: String::new(),
            item2: 0,
        }
    }
}
```

---

## 开发实施 (TDD)

### Phase 1: Red - 编写失败的测试

由于本步骤是结构初始化，测试重点是验证结构正确性：

```rust
// tests/integration/structure_test.rs

//! 项目结构验证测试

use std::path::Path;

/// 测试：验证核心模块目录存在
#[test]
fn test_core_modules_exist() {
    let modules = [
        "src/gateway",
        "src/validator",
        "src/bus",
        "src/memory",
        "src/audit",
    ];
    
    for module in &modules {
        assert!(
            Path::new(module).exists(),
            "核心模块目录不存在: {}",
            module
        );
    }
}

/// 测试：验证扩展模块目录存在
#[test]
fn test_extension_modules_exist() {
    let modules = [
        "src/sandbox",
        "src/skills",
        "src/mcp",
        "src/cli",
    ];
    
    for module in &modules {
        assert!(
            Path::new(module).exists(),
            "扩展模块目录不存在: {}",
            module
        );
    }
}

/// 测试：验证新增模块目录存在
#[test]
fn test_new_modules_exist() {
    let modules = [
        "src/rollback",
        "src/permission",
        "src/eventlog",
        "src/notifier",
        "src/bootstrap",
        "src/shutdown",
        "src/health",
        "src/transaction",
        "src/lock",
        "src/idempotency",
    ];
    
    for module in &modules {
        assert!(
            Path::new(module).exists(),
            "新增模块目录不存在: {}",
            module
        );
    }
}

/// 测试：验证文档目录结构
#[test]
fn test_docs_structure() {
    let paths = [
        "docs/tasks",
        "docs/reports",
        "docs/architecture",
        "docs/api",
    ];
    
    for path in &paths {
        assert!(
            Path::new(path).exists(),
            "文档目录不存在: {}",
            path
        );
    }
}

/// 测试：验证测试目录结构
#[test]
fn test_tests_structure() {
    let paths = [
        "tests/integration",
        "tests/e2e",
        "tests/common",
        "benches",
        "examples",
    ];
    
    for path in &paths {
        assert!(
            Path::new(path).exists(),
            "测试目录不存在: {}",
            path
        );
    }
}

/// 测试：验证配置文件存在
#[test]
fn test_config_files() {
    let files = [
        ".rustfmt.toml",
        "clippy.toml",
        ".gitignore",
    ];
    
    for file in &files {
        assert!(
            Path::new(file).exists(),
            "配置文件不存在: {}",
            file
        );
    }
}

/// 测试：验证Cargo工作空间
#[test]
fn test_cargo_workspace() {
    assert!(Path::new("Cargo.toml").exists());
    assert!(Path::new("Cargo.lock").exists());
}
```

### Phase 2: Green - 实现代码

创建必要的目录和文件：

```bash
# 创建新增模块目录
mkdir -p src/audit
mkdir -p src/rollback
mkdir -p src/permission
mkdir -p src/eventlog
mkdir -p src/notifier
mkdir -p src/bootstrap
mkdir -p src/shutdown
mkdir -p src/health
mkdir -p src/transaction
mkdir -p src/lock
mkdir -p src/idempotency
mkdir -p src/migration
mkdir -p src/backup
mkdir -p src/ratelimiter
mkdir -p src/plugin
mkdir -p src/schedule
mkdir -p src/cache

# 创建测试目录
mkdir -p tests/integration
mkdir -p tests/e2e
mkdir -p tests/common
mkdir -p benches
mkdir -p examples

# 创建文档目录
mkdir -p docs/tasks/phase-0
mkdir -p docs/reports
mkdir -p docs/architecture
mkdir -p docs/api
```

### Phase 3: Refactor - 优化重构

验证并优化：
- 配置文件规范性
- 目录命名一致性
- 模块骨架完整性

---

## 任务完成报告模板

完成任务后，需填写以下报告：

```markdown
# STEP-001 完成报告

## 完成内容清单

### 已创建目录
- [ ] 核心模块目录 (5个)
- [ ] 扩展模块目录 (4个)
- [ ] 新增模块目录 (12个)
- [ ] 测试目录 (5个)
- [ ] 文档目录 (4个)

### 已创建文件
- [ ] Cargo.toml (更新)
- [ ] .rustfmt.toml
- [ ] clippy.toml
- [ ] .gitignore (更新)
- [ ] 各模块骨架文件

## 测试执行结果

### 结构验证测试
```
test test_core_modules_exist ... ok
test test_extension_modules_exist ... ok
test test_new_modules_exist ... ok
test test_docs_structure ... ok
test test_tests_structure ... ok
test test_config_files ... ok
test test_cargo_workspace ... ok
```

### 编译验证
```
cargo check --workspace
    Checking gateway v0.1.0
    Checking validator v0.1.0
    ...
    Finished dev [unoptimized] target(s)
```

## 代码审查结果
- [ ] rustfmt检查通过
- [ ] clippy检查通过
- [ ] 无警告

## 遗留问题
（记录未解决的问题）

## 下一任务建议
- STEP-002: 开发环境配置
```

---

## 检查清单

### 任务开始前
- [x] 已阅读项目主控文档
- [x] 已确认项目愿景目标
- [x] 已确认当前执行步骤 (STEP-001)
- [x] 已确认项目完成度 (0%)
- [x] 已理解任务目标
- [x] 已了解技术约束
- [x] 已明确验收标准

### 任务完成时
- [ ] 所有测试通过
- [ ] cargo check通过
- [ ] rustfmt检查通过
- [ ] clippy检查通过
- [ ] 任务报告已编写
- [ ] 下一步骤已规划

---

## 附录

### A. 新增模块Cargo.toml模板

```toml
[package]
name = "fos-{module}"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
authors = ["FOS Team"]
description = "{模块描述}"
repository = "https://github.com/fos-platform/fos"

[lib]
name = "fos_{module}"
path = "src/lib.rs"

[dependencies]
fos-error = { path = "../error" }
tokio = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }

[lints]
workspace = true
```

### B. .rustfmt.toml 配置

```toml
max_width = 100
tab_spaces = 4
edition = "2021"
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
wrap_comments = true
format_code_in_doc_comments = true
```

### C. clippy.toml 配置

```toml
msrv = "1.75"
cognitive-complexity-threshold = 25
```

---

**下一步骤**: STEP-002 - 开发环境配置
