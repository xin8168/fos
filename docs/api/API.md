# FOS API 文档

## API 概述

FOS 提供统一的 RESTful API 和 gRPC 接口。

---

## 基础信息

| 项目 | 值 |
|-----|-----|
| 基础URL | http://localhost:8080/api/v1 |
| 协议 | HTTP/1.1, HTTP/2, gRPC |
| 数据格式 | JSON |
| 字符编码 | UTF-8 |
| 认证方式 | Bearer Token |

---

## 通用响应格式

### 成功响应

```json
{
  "code": 200,
  "message": "success",
  "data": { ... },
  "timestamp": 1700000000000
}
```

### 错误响应

```json
{
  "code": 400,
  "message": "错误描述",
  "error": {
    "type": "ValidationError",
    "details": "详细错误信息"
  },
  "timestamp": 1700000000000
}
```

---

## API 端点

### Gateway - 协议网关

#### POST /api/v1/gateway/validate

校验FOS命令格式

**请求体**:
```json
{
  "id": "cmd-001",
  "anchor": {
    "event": "事件名称",
    "steps": ["步骤1", "步骤2"],
    "judgment_logic": "判断逻辑",
    "verification_standard": "校验标准",
    "location": "执行地点",
    "subject": "执行主体"
  },
  "timestamp": 1700000000
}
```

**响应**:
```json
{
  "code": 200,
  "data": {
    "token": "合规令牌",
    "expires_at": "2026-03-09T01:00:00Z"
  }
}
```

#### GET /api/v1/gateway/health

健康检查

**响应**:
```json
{
  "code": 200,
  "data": {
    "status": "healthy",
    "uptime_secs": 3600
  }
}
```

---

### Validator - 规则校验

#### POST /api/v1/validator/validate

执行规则校验

**请求体**:
```json
{
  "request_id": "req-001",
  "event": "事件名称",
  "steps": ["步骤1"],
  "context": {
    "user_id": "user-001",
    "roles": ["admin"]
  }
}
```

**响应**:
```json
{
  "code": 200,
  "data": {
    "passed": true,
    "rule_results": []
  }
}
```

---

### Bus - 执行调度

#### POST /api/v1/bus/execute

执行任务

**请求体**:
```json
{
  "task": {
    "id": "task-001",
    "name": "任务名称",
    "steps": ["步骤1", "步骤2"],
    "priority": "normal"
  }
}
```

**响应**:
```json
{
  "code": 200,
  "data": {
    "task_id": "task-001",
    "status": "completed",
    "result": {
      "success": true,
      "output": "执行结果"
    }
  }
}
```

#### GET /api/v1/bus/tasks/{id}

查询任务状态

**响应**:
```json
{
  "code": 200,
  "data": {
    "id": "task-001",
    "status": "running",
    "progress": 50
  }
}
```

---

### Memory - 硬记忆库

#### GET /api/v1/memory/events

查询成功事件

**参数**:
- `name`: 事件名称（可选）
- `type`: 事件类型（可选）
- `limit`: 返回数量（默认20）

**响应**:
```json
{
  "code": 200,
  "data": {
    "events": [
      {
        "id": "evt-001",
        "name": "事件名称",
        "created_at": "2026-03-09T00:00:00Z"
      }
    ],
    "total": 100
  }
}
```

#### POST /api/v1/memory/reuse

复用成功事件

**请求体**:
```json
{
  "event_name": "事件名称"
}
```

---

### Audit - 拦截日志

#### GET /api/v1/audit/logs

查询拦截日志

**参数**:
- `log_type`: 日志类型（可选）
- `start_time`: 开始时间（可选）
- `end_time`: 结束时间（可选）

**响应**:
```json
{
  "code": 200,
  "data": {
    "logs": [
      {
        "id": "log-001",
        "log_type": "format_blocked",
        "reason": "格式错误",
        "created_at": "2026-03-09T00:00:00Z"
      }
    ]
  }
}
```

---

## 错误码

| 错误码 | 描述 |
|-------|------|
| 200 | 成功 |
| 400 | 请求参数错误 |
| 401 | 未授权 |
| 403 | 无权限 |
| 404 | 资源不存在 |
| 409 | 资源冲突 |
| 422 | 校验失败 |
| 500 | 内部错误 |
| 503 | 服务不可用 |

---

## 速率限制

| 端点 | 限制 |
|-----|------|
| 全局 | 1000 req/min |
| /execute | 100 req/min |
| /events | 500 req/min |

---

**版本**: v1.0.0  
**最后更新**: 2026-03-09
