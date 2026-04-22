"""
FOS CLI 数据模型

定义 FOS 命令和事件的数据结构
"""

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional
from uuid import uuid4

from pydantic import BaseModel, Field


class EventType(str, Enum):
    """事件类型"""

    DEVICE_CONTROL = "device_control"
    FILE_OPERATION = "file_operation"
    SYSTEM_COMMAND = "system_command"
    NETWORK_OPERATION = "network_operation"
    AUTOMATION_TASK = "automation_task"
    SKILL_EXECUTION = "skill_execution"


class EventStatus(str, Enum):
    """事件状态"""

    PENDING = "pending"
    VALIDATING = "validating"
    EXECUTING = "executing"
    SUCCESS = "success"
    FAILED = "failed"
    BLOCKED = "blocked"
    ROLLED_BACK = "rolled_back"


class SixAnchor(BaseModel):
    """FOS 6维锚定结构"""

    event: str = Field(..., description="事件：用户想要达成的目标")
    steps: List[str] = Field(..., description="步骤：明确的执行步骤序列")
    judgment_logic: str = Field(..., description="判断逻辑：每一步的判断条件")
    verification_standard: str = Field(..., description="校验标准：最终结果的成功标准")
    location: str = Field(..., description="地点：执行环境/设备")
    subject: str = Field(..., description="主体：执行者身份")


class FourElement(BaseModel):
    """FOS 4要素执行结构"""

    action: str = Field(..., description="动作：具体的操作行为")
    target: str = Field(..., description="对象：操作的目标对象")
    condition: str = Field(..., description="条件：执行前提条件")
    expected_result: str = Field(..., description="结果：预期的执行结果")


class FosCommand(BaseModel):
    """FOS 完整命令结构"""

    id: str = Field(default_factory=lambda: str(uuid4()), description="命令唯一ID")
    anchor: SixAnchor = Field(..., description="6维锚定")
    elements: List[FourElement] = Field(default_factory=list, description="4要素执行列表")
    timestamp: int = Field(
        default_factory=lambda: int(datetime.now().timestamp() * 1000),
        description="时间戳",
    )
    metadata: Dict[str, str] = Field(default_factory=dict, description="元数据")


class StepResult(BaseModel):
    """步骤执行结果"""

    step_index: int = Field(..., description="步骤索引")
    success: bool = Field(..., description="是否成功")
    output: str = Field(default="", description="输出")
    error: Optional[str] = Field(default=None, description="错误信息")


class ExecutionResult(BaseModel):
    """执行结果"""

    success: bool = Field(..., description="是否成功")
    output: str = Field(default="", description="输出内容")
    error: Optional[str] = Field(default=None, description="错误信息")
    duration_ms: int = Field(default=0, description="执行时间（毫秒）")
    step_results: List[StepResult] = Field(default_factory=list, description="步骤结果")


class FosEvent(BaseModel):
    """FOS 事件结构"""

    id: str = Field(default_factory=lambda: str(uuid4()), description="事件ID")
    name: str = Field(..., description="事件名称")
    event_type: EventType = Field(..., description="事件类型")
    status: EventStatus = Field(default=EventStatus.PENDING, description="事件状态")
    anchor: SixAnchor = Field(..., description="6维锚定")
    result: Optional[ExecutionResult] = Field(default=None, description="执行结果")
    created_at: int = Field(
        default_factory=lambda: int(datetime.now().timestamp() * 1000),
        description="创建时间",
    )
    updated_at: int = Field(
        default_factory=lambda: int(datetime.now().timestamp() * 1000),
        description="更新时间",
    )


class EventQuery(BaseModel):
    """事件查询条件"""

    name: Optional[str] = Field(default=None, description="事件名称（模糊匹配）")
    event_type: Optional[EventType] = Field(default=None, description="事件类型")
    location: Optional[str] = Field(default=None, description="执行地点")
    subject: Optional[str] = Field(default=None, description="执行主体")
    start_time: Optional[int] = Field(default=None, description="开始时间")
    end_time: Optional[int] = Field(default=None, description="结束时间")
    offset: int = Field(default=0, description="分页偏移")
    limit: int = Field(default=20, description="分页限制")


class ApiResponse(BaseModel):
    """API 响应"""

    success: bool = Field(..., description="是否成功")
    code: int = Field(default=200, description="响应码")
    message: str = Field(default="成功", description="消息")
    data: Optional[Dict[str, Any]] = Field(default=None, description="数据")

    class Config:
        """Pydantic 配置"""

        json_schema_extra = {
            "example": {
                "success": True,
                "code": 200,
                "message": "成功",
                "data": {"event_id": "xxx", "status": "pending"},
            }
        }
