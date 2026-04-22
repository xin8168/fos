# FOS CLI - 命令行界面模块

FOS 明文交互入口，用户与 FOS 的唯一人机交互层

## 核心职责
- 接收中文明文指令
- 解析和验证指令格式
- 与 Gateway 通信执行任务
- 实时显示执行状态

## 安装

```bash
pip install fos-cli
```

## 使用

```bash
# 执行任务
fos run --事件 "清理电脑桌面无用文件" --步骤 "列出桌面文件，筛选7天未修改文件，移动到归档文件夹"

# 复用成功事件
fos reuse "清理电脑桌面无用文件"

# 查看事件列表
fos list --type device_control

# 查看事件详情
fos show <event_id>
```

## 配置

环境变量:
- FOS_GATEWAY_URL: Gateway 服务地址 (默认: http://localhost:8080)
- FOS_API_KEY: API 密钥
- FOS_TIMEOUT: 请求超时时间（秒）
