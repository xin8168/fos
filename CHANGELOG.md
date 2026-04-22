# Changelog

All notable changes to **FOS神经元控制器** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.1.0] - 2026-04-22

### 重大更新

**项目重命名**: FOS确定性执行主板 → **FOS神经元控制器**

**新定位**: AI能力延展的核心执行控制框架，专为大模型能力延展、脑机能力延展、具身智能控制而设计。

### Added

#### 核心模块 (27个模块)

**感觉神经层**
- **fos-gateway** - 神经信号输入层 (18测试)
  - FosValidator信号格式校验
  - 令牌管理
  - 协议解析

**脊髓神经层**
- **fos-validator** - 安全边界校验层 (20测试)
  - 红线规则引擎
  - 权限校验
  - 环境验证

**运动神经层**
- **fos-bus** - 执行调度层 (42测试)
  - 优先级队列
  - 超时管理
  - 重试机制

**神经末梢层**
- **fos-mcp** - 设备控制接口 (44测试)
  - 设备注册与管理
  - 心跳监控
  - 离线缓存

**记忆与反馈**
- **fos-memory** - 情景记忆存储 (22测试)
- **fos-audit** - 安全反馈记录 (18测试)
- **fos-eventlog** - 链路追踪 (27测试)

**辅助神经模块**
- **fos-bootstrap** - 神经发育初始化 (17测试)
  - 五阶段启动: 配置→存储→核心→扩展→服务
- **fos-health** - 本体感觉健康检查 (31测试)
  - 依赖检查、自愈机制
- **fos-lock** - 肌肉张力分布式锁 (31测试)
  - 可重入锁、公平等待
- **fos-transaction** - 协同运动事务 (34测试)
  - Saga模式、补偿执行
- **fos-idempotency** - 神经记忆幂等 (29测试)
  - 信号去重、状态机
- **fos-sandbox** - 反射弧隔离 (27测试)
  - 文件/网络/进程隔离
- **fos-rollback** - 撤回反射 (44测试)
  - 快照管理、原子回滚

**共享类型库**
- **fos-common** - 核心信号类型 (20测试)
  - SixAnchor六维锚定
  - FourElement四要素
  - FosCommand神经信号
  - FosMessage跨层通信

#### 应用场景支持

| 场景 | 模块支持 | 说明 |
|------|--------|------|
| 大模型能力延展 | Gateway + Validator + Bus | 决策→执行闭环 |
| 脑机能力延展 | MCP + Lock | BCI信号→设备映射 |
| 具身智能控制 | Skills + MCP + Sandbox | 意图→动作执行 |
| AI Agent执行 | Transaction + Idempotency | 规划→验证→回滚 |

#### 部署配置

- **Dockerfile** - 多阶段构建(cargo-chef)
- **K8s部署栈** - 完整Kubernetes部署
  - Deployment(Gateway/Validator/Bus)
  - StatefulSet(PostgreSQL/Redis)
  - NetworkPolicy(零信任网络)
- **监控配置** - Prometheus + Grafana + Fluent Bit

#### 文档

- README.md - 项目概览(已更新为神经元控制器)
- PRODUCT_SUMMARY.md - 产品技术白皮书(已更新)
- ARCHITECTURE.md - 架构设计文档
- USER_GUIDE.md - 用户使用指南
- DEVELOPMENT.md - 开发手册

---

## [v0.0.1] - 2026-03-09

### Added

- 初始项目结构
- 120个开发步骤跟踪
- 核心模块占位符

---

## 技术演进

### 传统执行 vs 神经元控制

```
传统执行架构:
LLM → API → 执行器 → 结果
         ↓
      不可靠

神经元控制器架构:
LLM → 感觉神经(Gateway) → 脊髓神经(Validator) → 运动神经(Bus) → 神经末梢(MCP) → 设备
         ↓                    ↓                    ↓                    ↓
      信号标准化          安全校验              确定性执行           精确控制
```

---

## 安全铁律

1. **信号链唯一**: 所有信号必须通过感觉→脊髓→运动三层
2. **脊髓不可绕过**: 脊髓校验不可跳过
3. **反射弧隔离**: 危险动作反射执行
4. **反馈必闭环**: 执行结果必须反馈

---

## Credits

- FOS Team - 核心开发团队
- 所有贡献者

---

## License

This project is licensed under the Apache-2.0 License - see the LICENSE file for details.

**FOS - 让AI的每一个决策，都精确到神经末梢**