# FOS 任务索引

## 文档结构

```
docs/
├── FOS-MASTER-CONTROL.md      # 主控文档（必须阅读）
├── CORE-SUMMARY.md            # 核心摘要（压缩时保留）
├── FOS-产品开发任务书_v1.0.md  # 产品需求文档
├── PROGRESS.md                # 进度跟踪
├── TASK-INDEX.md              # 本文档（任务索引）
├── tasks/                     # 任务文档目录
│   ├── phase-0/              # Phase 0 任务
│   ├── phase-1/              # Phase 1 任务
│   ├── phase-2/              # Phase 2 任务
│   ├── phase-3/              # Phase 3 任务
│   ├── phase-4/              # Phase 4 任务
│   ├── phase-5/              # Phase 5 任务
│   ├── phase-6/              # Phase 6 任务
│   ├── phase-7/              # Phase 7 任务
│   └── phase-8/              # Phase 8 任务
└── reports/                   # 完成报告目录
```

---

## 任务总览

| Phase | 名称 | 步骤范围 | 模块 | 状态 |
|-------|------|---------|------|------|
| 0 | 基础设施搭建 | 001-010 | Bootstrap/Shutdown/Health | 🔄 进行中 |
| 1 | 数据一致性模块 | 011-020 | Transaction/Lock/Idempotency | ⏳ 待执行 |
| 2 | 核心模块完善 | 021-040 | Gateway/Validator/Bus/Memory/Audit | ⏳ 待执行 |
| 3 | 新增核心模块 | 041-060 | Rollback/Permission/EventLog | ⏳ 待执行 |
| 4 | 扩展模块完善 | 061-080 | Sandbox/Skills/MCP/CLI | ⏳ 待执行 |
| 5 | 运维支持模块 | 081-090 | Migration/Backup/RateLimiter | ⏳ 待执行 |
| 6 | 扩展能力模块 | 091-100 | Plugin/Schedule/Cache | ⏳ 待执行 |
| 7 | 系统集成测试 | 101-110 | 全链路测试 | ⏳ 待执行 |
| 8 | 部署与交付 | 111-120 | 部署/监控/文档 | ⏳ 待执行 |

---

## Phase 0: 基础设施搭建

| 步骤 | 任务名称 | 文档路径 | 状态 |
|-----|---------|---------|------|
| 001 | 项目结构初始化 | docs/tasks/phase-0/STEP-001.md | 🔄 待执行 |
| 002 | 开发环境配置 | docs/tasks/phase-0/STEP-002.md | ⏳ 待创建 |
| 003 | CI/CD流水线搭建 | docs/tasks/phase-0/STEP-003.md | ⏳ 待创建 |
| 004 | 测试框架搭建 | docs/tasks/phase-0/STEP-004.md | ⏳ 待创建 |
| 005 | 文档系统搭建 | docs/tasks/phase-0/STEP-005.md | ⏳ 待创建 |
| 006 | Bootstrap启动引导 | docs/tasks/phase-0/STEP-006.md | ⏳ 待创建 |
| 007 | Shutdown优雅关闭 | docs/tasks/phase-0/STEP-007.md | ⏳ 待创建 |
| 008 | HealthCheck健康检查 | docs/tasks/phase-0/STEP-008.md | ⏳ 待创建 |
| 009 | 配置管理完善 | docs/tasks/phase-0/STEP-009.md | ⏳ 待创建 |
| 010 | 基础设施集成测试 | docs/tasks/phase-0/STEP-010.md | ⏳ 待创建 |

---

## 快速导航

### 开始新任务
1. 阅读 `docs/FOS-MASTER-CONTROL.md`
2. 确认项目愿景和当前步骤
3. 阅读对应步骤的任务文档
4. 执行TDD开发流程

### 查看进度
- 查看 `docs/PROGRESS.md`

### 核心信息回顾
- 查看 `docs/CORE-SUMMARY.md`

---

## 任务执行流程

```
┌─────────────────────────────────────────────────────────────┐
│                     任务执行流程                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. 阅读主控文档                                            │
│     └─→ docs/FOS-MASTER-CONTROL.md                         │
│                                                             │
│  2. 确认项目愿景                                            │
│     └─→ 零风险、零兜底、零混乱                              │
│                                                             │
│  3. 确认当前步骤                                            │
│     └─→ docs/PROGRESS.md                                   │
│                                                             │
│  4. 阅读任务文档                                            │
│     └─→ docs/tasks/phase-X/STEP-XXX.md                     │
│                                                             │
│  5. 执行TDD开发                                             │
│     ├─→ Red: 编写失败测试                                   │
│     ├─→ Green: 实现最小代码                                 │
│     └─→ Refactor: 优化重构                                  │
│                                                             │
│  6. 验证绿灯状态                                            │
│     ├─→ cargo test --workspace                             │
│     ├─→ cargo fmt --check                                  │
│     ├─→ cargo clippy -- -D warnings                        │
│     └─→ cargo tarpaulin (覆盖率≥90%)                       │
│                                                             │
│  7. 编写完成报告                                            │
│     └─→ docs/reports/STEP-XXX-report.md                    │
│                                                             │
│  8. 更新进度                                                │
│     └─→ docs/PROGRESS.md                                   │
│                                                             │
│  9. 自动启动下一任务                                        │
│     └─→ 返回步骤1                                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 验收检查清单

### 任务开始前
- [ ] 已阅读主控文档
- [ ] 已确认项目愿景
- [ ] 已确认当前步骤
- [ ] 已阅读任务文档

### 任务完成时
- [ ] 所有测试通过
- [ ] 代码格式正确
- [ ] Clippy检查通过
- [ ] 覆盖率达标
- [ ] 无安全漏洞
- [ ] 文档已更新
- [ ] 完成报告已编写

---

**最后更新**: 2026-03-09
