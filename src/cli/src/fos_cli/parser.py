"""
FOS CLI 命令解析器

提供命令行参数解析、验证和转换功能
"""

import json
import os
import re
import shlex
import time
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

from pydantic import BaseModel, Field, ValidationError


class ParamType(str, Enum):
    """参数类型"""

    STRING = "string"
    INTEGER = "integer"
    FLOAT = "float"
    BOOLEAN = "boolean"
    JSON = "json"
    FILE = "file"
    LIST = "list"
    DATE = "date"


@dataclass
class CommandParam:
    """命令参数定义"""

    name: str
    short_name: Optional[str] = None
    long_name: Optional[str] = None
    param_type: ParamType = ParamType.STRING
    required: bool = False
    default: Any = None
    description: str = ""
    choices: Optional[List[str]] = None
    validator: Optional[Callable[[Any], bool]] = None
    env_var: Optional[str] = None


@dataclass
class CommandDefinition:
    """命令定义"""

    name: str
    aliases: List[str] = field(default_factory=list)
    description: str = ""
    params: List[CommandParam] = field(default_factory=list)
    handler: Optional[Callable] = None
    category: str = "general"


class CommandParseError(Exception):
    """命令解析错误"""

    def __init__(self, message: str, suggestions: Optional[List[str]] = None):
        self.message = message
        self.suggestions = suggestions or []
        super().__init__(message)


class CommandParser:
    """命令解析器"""

    def __init__(self):
        """初始化命令解析器"""
        self.commands: Dict[str, CommandDefinition] = {}
        self.categories: Dict[str, List[str]] = {}

    def register_command(self, command: CommandDefinition) -> None:
        """注册命令"""
        self.commands[command.name] = command

        # 注册别名
        for alias in command.aliases:
            self.commands[alias] = command

        # 分类
        if command.category not in self.categories:
            self.categories[command.category] = []
        self.categories[command.category].append(command.name)

    def parse_args(
        self,
        command_line: Union[str, List[str]],
        command_name: Optional[str] = None,
    ) -> Tuple[str, Dict[str, Any]]:
        """
        解析命令行参数

        Args:
            command_line: 命令行字符串或参数列表
            command_name: 命令名称（如果未提供则从command_line中解析）

        Returns:
            (命令名, 参数字典)

        Raises:
            CommandParseError: 解析错误
        """
        # 统一转换为列表
        if isinstance(command_line, str):
            args = shlex.split(command_line)
        else:
            args = command_line.copy()

        # 解析命令名
        if command_name is None:
            if not args:
                raise CommandParseError("缺少命令名")

            command_name = args.pop(0)

        # 获取命令定义
        command_def = self.commands.get(command_name)
        if command_def is None:
            suggestions = self._suggest_command(command_name)
            raise CommandParseError(
                f"未知命令 '{command_name}'", suggestions=suggestions
            )

        # 解析参数
        parsed_params = self._parse_params(command_def, args)

        return command_name, parsed_params

    def _parse_params(
        self, command_def: CommandDefinition, args: List[str]
    ) -> Dict[str, Any]:
        """
        解析命令参数

        Args:
            command_def: 命令定义
            args: 剩余参数列表

        Returns:
            参数字典
        """
        params = {}
        i = 0

        while i < len(args):
            arg = args[i]

            # 检查是否为参数标志
            if arg.startswith("-"):
                param_name = self._parse_param_flag(arg, command_def)

                # 查找参数定义
                param_def = self._find_param_def(command_def, param_name)

                # 解析参数值
                if param_def.param_type == ParamType.BOOLEAN:
                    # 布尔型参数不需要值
                    param_value = True
                else:
                    # 需要值的参数
                    if i + 1 >= len(args) or args[i + 1].startswith("-"):
                        raise CommandParseError(
                            f"参数 '{param_name}' 需要值"
                        )
                    i += 1
                    param_value = self._parse_param_value(
                        args[i], param_def, command_def
                    )

                params[param_name] = param_value

            else:
                # 位置参数
                # 检查是否有未解析的必需参数
                remaining_required = [
                    p for p in command_def.params if p.required and p.name not in params
                ]
                if remaining_required:
                    param_def = remaining_required[0]
                    param_value = self._parse_param_value(arg, param_def, command_def)
                    params[param_def.name] = param_value
                else:
                    raise CommandParseError(
                        f"意外的参数: {arg}，可能使用了错误的参数名"
                    )

            i += 1

        # 从环境变量读取
        for param_def in command_def.params:
            if (
                param_def.name not in params
                and param_def.env_var
                and param_def.env_var in os.environ
            ):
                params[param_def.name] = self._parse_param_value(
                    os.environ[param_def.env_var], param_def, command_def
                )

        # 设置默认值
        for param_def in command_def.params:
            if param_def.name not in params:
                if param_def.default is not None:
                    params[param_def.name] = param_def.default
                elif param_def.required:
                    raise CommandParseError(
                        f"缺少必需参数: {self._format_param_name(param_def)}"
                    )

        # 验证参数
        self._validate_params(command_def, params)

        return params

    def _parse_param_flag(self, flag: str, command_def: CommandDefinition) -> str:
        """
        解析参数标志

        Args:
            flag: 参数标志（如 -e, --事件, --event）
            command_def: 命令定义

        Returns:
            参数名称
        """
        # 长选项：--event, --事件
        if flag.startswith("--"):
            long_flag = flag[2:]

            # 查找匹配的参数
            for param in command_def.params:
                if param.long_name == long_flag:
                    return param.name

            # 查找中文名称
            for param in command_def.params:
                if param.name == long_flag:
                    return param.name

        # 短选项：-e
        elif flag.startswith("-"):
            short_flag = flag[1:]

            # 查找匹配的参数
            for param in command_def.params:
                if param.short_name == short_flag:
                    return param.name

        raise CommandParseError(f"未知参数 '{flag}'")

    def _find_param_def(
        self, command_def: CommandDefinition, param_name: str
    ) -> CommandParam:
        """查找参数定义"""
        for param in command_def.params:
            if param.name == param_name:
                return param

        raise CommandParseError(f"参数 '{param_name}' 未定义")

    def _parse_param_value(
        self,
        value: str,
        param_def: CommandParam,
        command_def: CommandDefinition,
    ) -> Any:
        """
        解析参数值

        Args:
            value: 值字符串
            param_def: 参数定义
            command_def: 命令定义

        Returns:
            解析后的值
        """
        try:
            if param_def.param_type == ParamType.STRING:
                return value

            elif param_def.param_type == ParamType.INTEGER:
                return int(value)

            elif param_def.param_type == ParamType.FLOAT:
                return float(value)

            elif param_def.param_type == ParamType.BOOLEAN:
                lower_value = value.lower()
                if lower_value in ("true", "yes", "1", "on"):
                    return True
                elif lower_value in ("false", "no", "0", "off"):
                    return False
                else:
                    raise ValueError(f"无效的布尔值: {value}")

            elif param_def.param_type == ParamType.LIST:
                # 支持逗号分隔的列表
                items = [item.strip() for item in value.split(",")]
                return items

            elif param_def.param_type == ParamType.FILE:
                # 验证文件路径
                file_path = Path(value)
                if not file_path.exists():
                    raise ValueError(f"文件不存在: {value}")
                return file_path

            elif param_def.param_type == ParamType.DATE:
                # 简单日期解析（ISO格式）
                from datetime import datetime as dt

                return dt.fromisoformat(value)

            elif param_def.param_type == ParamType.JSON:
                # JSON 解析
                import json

                return json.loads(value)

            else:
                return value

        except (ValueError, json.JSONDecodeError) as e:
            raise CommandParseError(
                f"参数 '{param_def.name}' 的值 '{value}' 格式错误: {e}"
            )

    def _validate_params(
        self, command_def: CommandDefinition, params: Dict[str, Any]
    ) -> None:
        """验证参数"""
        for param_def in command_def.params:
            if param_def.name not in params:
                continue

            value = params[param_def.name]

            # 验证选择项
            if param_def.choices and value not in param_def.choices:
                raise CommandParseError(
                    f"参数 '{param_def.name}' 的值 '{value}' 不在允许的选项中: {param_def.choices}"
                )

            # 自定义验证
            if param_def.validator:
                try:
                    if not param_def.validator(value):
                        raise CommandParseError(
                            f"参数 '{param_def.name}' 的值 '{value}' 验证失败"
                        )
                except Exception as e:
                    raise CommandParseError(
                        f"参数 '{param_def.name}' 验证错误: {e}"
                    )

    def _format_param_name(self, param_def: CommandParam) -> str:
        """格式化参数名称显示"""
        parts = []
        if param_def.short_name:
            parts.append(f"-{param_def.short_name}")
        if param_def.long_name:
            parts.append(f"--{param_def.long_name}")
        elif param_def.name:
            parts.append(f"--{param_def.name}")
        return ", ".join(parts)

    def _suggest_command(self, command_name: str) -> List[str]:
        """
        建议相似的命令名

        Args:
            command_name: 错误的命令名

        Returns:
            建议的命令名列表（最多3个）
        """
        suggestions = []
        threshold = 2  # 编辑距离阈值

        for registered_name in self.commands.keys():
            distance = self._levenshtein_distance(command_name, registered_name)
            if distance <= threshold:
                suggestions.append(registered_name)

        # 去重并排序
        suggestions = list(set(suggestions))[:3]
        suggestions.sort(key=lambda x: self._levenshtein_distance(command_name, x))

        return suggestions

    def _levenshtein_distance(self, s1: str, s2: str) -> int:
        """计算编辑距离"""
        if len(s1) < len(s2):
            return self._levenshtein_distance(s2, s1)

        if len(s2) == 0:
            return len(s1)

        previous_row = range(len(s2) + 1)
        for i, c1 in enumerate(s1):
            current_row = [i + 1]
            for j, c2 in enumerate(s2):
                insertions = previous_row[j + 1] + 1
                deletions = current_row[j] + 1
                substitutions = previous_row[j] + (c1 != c2)
                current_row.append(min(insertions, deletions, substitutions))
            previous_row = current_row

        return previous_row[-1]

    def get_command_help(self, command_name: str) -> str:
        """获取命令帮助信息"""
        command_def = self.commands.get(command_name)
        if command_def is None:
            raise CommandParseError(f"未知命令 '{command_name}'")

        help_lines = []
        help_lines.append(f"命令: {command_def.name}")
        if command_def.aliases:
            help_lines.append(f"别名: {', '.join(command_def.aliases)}")
        help_lines.append(f"描述: {command_def.description}")
        help_lines.append("")
        help_lines.append("参数:")

        for param in command_def.params:
            required = " [必需]" if param.required else " [可选]"
            param_line = f"  {self._format_param_name(param)}{required}"
            help_lines.append(param_line)
            help_lines.append(f"    类型: {param.param_type.value}")
            if param.default is not None:
                help_lines.append(f"    默认: {param.default}")
            if param.description:
                help_lines.append(f"    描述: {param.description}")
            if param.choices:
                help_lines.append(f"    选项: {', '.join(param.choices)}")
            if param.env_var:
                help_lines.append(f"    环境变量: {param.env_var}")
            help_lines.append("")

        return "\n".join(help_lines)

    def list_commands(self, category: Optional[str] = None) -> Dict[str, List[str]]:
        """列出所有命令"""
        if category:
            return {category: self.categories.get(category, [])}
        return self.categories


class ParsedCommand(BaseModel):
    """解析后的命令"""

    command: str = Field(..., description="命令名称")
    params: Dict[str, Any] = Field(default_factory=dict, description="参数")
    raw_args: List[str] = Field(default_factory=list, description="原始参数列表")
    parsed_at: int = Field(
        default_factory=lambda: int(time.time()),
        description="解析时间",
    )


class CommandLineParser:
    """完整的命令行解析器（支持子命令）"""

    def __init__(self):
        """初始化"""
        self.parser = CommandParser()
        self.global_params: List[CommandParam] = []

    def add_global_param(self, param: CommandParam) -> None:
        """添加全局参数"""
        self.global_params.append(param)

    def register_command(self, command: CommandDefinition) -> None:
        """注册命令"""
        self.parser.register_command(command)

    def parse(self, command_line: Union[str, List[str]]) -> ParsedCommand:
        """
        解析完整命令行

        Args:
            command_line: 命令行

        Returns:
            解析后的命令
        """
        # 解析全局参数（简化版，只提取--config等）
        if isinstance(command_line, str):
            args = shlex.split(command_line)
        else:
            args = command_line.copy()

        # 解析命令和参数
        command_name, params = self.parser.parse_args(args)

        return ParsedCommand(command=command_name, params=params, raw_args=args)


def default_parser() -> CommandLineParser:
    """创建默认的命令行解析器"""
    parser = CommandLineParser()

    # 添加全局参数
    parser.add_global_param(
        CommandParam(
            name="config",
            long_name="config",
            short_name="c",
            param_type=ParamType.FILE,
            description="配置文件路径",
            env_var="FOS_CONFIG",
        )
    )

    parser.add_global_param(
        CommandParam(
            name="verbose",
            long_name="verbose",
            short_name="v",
            param_type=ParamType.BOOLEAN,
            description="详细输出",
            env_var="FOS_VERBOSE",
        )
    )

    parser.add_global_param(
        CommandParam(
            name="timeout",
            long_name="timeout",
            short_name="t",
            param_type=ParamType.INTEGER,
            description="超时时间（秒）",
            default=30,
            env_var="FOS_TIMEOUT",
        )
    )

    return parser
