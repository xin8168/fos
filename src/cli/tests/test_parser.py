"""
测试命令行解析器

测试命令参数解析、验证和转换功能
"""

import os
import tempfile
from pathlib import Path

import pytest

from fos_cli.parser import (
    CommandDefinition,
    CommandLineParser,
    CommandParam,
    CommandParseError,
    CommandParser,
    ParamType,
    ParsedCommand,
    default_parser,
)


class TestCommandParser:
    """测试命令解析器"""

    @pytest.fixture
    def parser(self):
        """创建测试解析器"""
        return CommandParser()

    @pytest.fixture
    def sample_command(self):
        """创建示例命令"""
        return CommandDefinition(
            name="test",
            description="测试命令",
            params=[
                CommandParam(
                    name="event",
                    short_name="e",
                    long_name="事件",
                    param_type=ParamType.STRING,
                    required=True,
                    description="事件名称",
                ),
                CommandParam(
                    name="steps",
                    short_name="s",
                    long_name="步骤",
                    param_type=ParamType.LIST,
                    description="执行步骤",
                    default=[],
                ),
                CommandParam(
                    name="verbose",
                    short_name="v",
                    param_type=ParamType.BOOLEAN,
                    description="详细输出",
                ),
                CommandParam(
                    name="count",
                    short_name="n",
                    param_type=ParamType.INTEGER,
                    description="数量",
                    default=10,
                ),
            ],
        )

    def test_register_command(self, parser, sample_command):
        """测试命令注册"""
        parser.register_command(sample_command)

        assert "test" in parser.commands
        assert parser.commands["test"] == sample_command

    def test_parse_required_param(self, parser, sample_command):
        """测试解析必需参数"""
        parser.register_command(sample_command)

        command_name, params = parser.parse_args("test --事件 my_event")

        assert command_name == "test"
        assert params["event"] == "my_event"

    def test_parse_short_param(self, parser, sample_command):
        """测试解析短参数"""
        parser.register_command(sample_command)

        command_name, params = parser.parse_args("test -e my_event")

        assert params["event"] == "my_event"

    def test_parse_multiple_params(self, parser, sample_command):
        """测试解析多个参数"""
        parser.register_command(sample_command)

        command_name, params = parser.parse_args("test -e my_event -n 20")

        assert params["event"] == "my_event"
        assert params["count"] == 20

    def test_parse_list_param(self, parser, sample_command):
        """测试解析列表参数"""
        parser.register_command(sample_command)

        command_name, params = parser.parse_args(
            "test --事件 my_event --步骤 step1,step2,step3"
        )

        assert params["steps"] == ["step1", "step2", "step3"]

    def test_parse_boolean_param(self, parser, sample_command):
        """测试解析布尔参数"""
        parser.register_command(sample_command)

        command_name, params = parser.parse_args("test -e my_event -v")

        assert params["verbose"] is True

    def test_missing_required_param(self, parser, sample_command):
        """测试缺少必需参数"""
        parser.register_command(sample_command)

        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("test -n 10")

        assert "缺少必需参数" in str(exc_info.value)

    def test_unknown_param(self, parser, sample_command):
        """测试未知参数"""
        parser.register_command(sample_command)

        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("test -e my_event --unknown value")

        assert "未知参数" in str(exc_info.value)

    def test_unknown_command(self, parser):
        """测试未知命令"""
        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("unknown_command")

        assert "未知命令" in str(exc_info.value)
        # 验证是否提供了建议
        assert exc_info.value.suggestions is not None

    def test_param_choices_validation(self, parser):
        """测试参数选择验证"""
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(
                    name="action",
                    choices=["start", "stop", "restart"],
                    required=True,
                )
            ],
        )
        parser.register_command(command)

        # 有效选择
        _, params = parser.parse_args("test --action start")
        assert params["action"] == "start"

        # 无效选择
        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("test --action pause")
        assert "不在允许的选项中" in str(exc_info.value)

    def test_param_with_file_type(self, parser):
        """测试文件类型参数"""
        # 创建临时文件
        with tempfile.NamedTemporaryFile(delete=False) as f:
            temp_file = f.name
            f.write(b"test content")

        try:
            command = CommandDefinition(
                name="test",
                params=[
                    CommandParam(name="config", param_type=ParamType.FILE, required=True)
                ],
            )
            parser.register_command(command)

            _, params = parser.parse_args(f"test --config {temp_file}")
            assert params["config"] == Path(temp_file)
        finally:
            os.unlink(temp_file)

    def test_param_with_invalid_file(self, parser):
        """测试无效文件路径"""
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="config", param_type=ParamType.FILE, required=True)
            ],
        )
        parser.register_command(command)

        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("test --config /nonexistent/file.txt")
        assert "文件不存在" in str(exc_info.value)

    def test_custom_validator(self, parser):
        """测试自定义验证器"""
        def validate_port(value):
            return 0 < value < 65536

        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(
                    name="port",
                    param_type=ParamType.INTEGER,
                    validator=validate_port,
                    required=True,
                )
            ],
        )
        parser.register_command(command)

        # 有效端口
        _, params = parser.parse_args("test --port 8080")
        assert params["port"] == 8080

        # 无效端口
        with pytest.raises(CommandParseError) as exc_info:
            parser.parse_args("test --port 99999")
        assert "验证失败" in str(exc_info.value)

    def test_command_aliases(self, parser):
        """测试命令别名"""
        command = CommandDefinition(
            name="execute",
            aliases=["run", "exec"],
            params=[CommandParam(name="cmd", required=True)],
        )
        parser.register_command(command)

        # 使用主命令名
        _, params1 = parser.parse_args("execute --cmd ls")
        assert params1["cmd"] == "ls"

        # 使用别名
        _, params2 = parser.parse_args("run --cmd ls")
        assert params2["cmd"] == "ls"

        _, params3 = parser.parse_args("exec --cmd ls")
        assert params3["cmd"] == "ls"

    def test_default_values(self, parser, sample_command):
        """测试默认值"""
        parser.register_command(sample_command)

        _, params = parser.parse_args("test -e my_event")

        assert params["count"] == 10
        assert params["steps"] == []

    def test_env_var_fallback(self, parser):
        """测试环境变量回退"""
        os.environ["TEST_EVENT"] = "env_event"

        try:
            command = CommandDefinition(
                name="test",
                params=[
                    CommandParam(
                        name="event",
                        env_var="TEST_EVENT",
                        required=True,
                    )
                ],
            )
            parser.register_command(command)

            _, params = parser.parse_args("test")
            assert params["event"] == "env_event"
        finally:
            os.environ.pop("TEST_EVENT", None)

    def test_positional_params(self, parser):
        """测试位置参数"""
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="arg1", required=True),
                CommandParam(name="arg2", required=True),
            ],
        )
        parser.register_command(command)

        _, params = parser.parse_args("test value1 value2")
        assert params["arg1"] == "value1"
        assert params["arg2"] == "value2"

    def test_get_command_help(self, parser, sample_command):
        """测试获取命令帮助"""
        parser.register_command(sample_command)

        help_text = parser.get_command_help("test")

        assert "命令: test" in help_text
        assert "参数:" in help_text
        assert "事件名称" in help_text

    def test_list_commands(self, parser):
        """测试列出命令"""
        cmd1 = CommandDefinition(name="cmd1", category="general")
        cmd2 = CommandDefinition(name="cmd2", category="general")
        cmd3 = CommandDefinition(name="cmd3", category="advanced")

        parser.register_command(cmd1)
        parser.register_command(cmd2)
        parser.register_command(cmd3)

        categories = parser.list_commands()

        assert "general" in categories
        assert "advanced" in categories
        assert len(categories["general"]) == 2
        assert len(categories["advanced"]) == 1


class TestCommandLineParser:
    """测试完整命令行解析器"""

    @pytest.fixture
    def parser(self):
        """创建测试解析器"""
        return CommandLineParser()

    def test_parse_with_global_params(self, parser):
        """测试解析包含全局参数的命令"""
        # 注册测试命令
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="event", required=True, param_type=ParamType.STRING)
            ],
        )
        parser.register_command(command)

        parsed = parser.parse("test --event my_event --verbose")

        assert parsed.command == "test"
        assert parsed.params["event"] == "my_event"

    def test_default_parser(self):
        """测试默认解析器"""
        parser = default_parser()

        # 注册测试命令
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="cmd", required=True, param_type=ParamType.STRING)
            ],
        )
        parser.register_command(command)

        parsed = parser.parse("test --cmd ls --timeout 60")

        assert parsed.command == "test"
        assert parsed.params["cmd"] == "ls"


class TestParsedCommand:
    """测试解析后的命令"""

    def test_create_parsed_command(self):
        """测试创建解析后的命令"""
        parsed = ParsedCommand(
            command="test",
            params={"param1": "value1"},
            raw_args=["test", "--param1", "value1"],
        )

        assert parsed.command == "test"
        assert parsed.params["param1"] == "value1"
        assert len(parsed.raw_args) == 3

    def test_to_dict(self):
        """测试转换为字典"""
        parsed = ParsedCommand(
            command="test",
            params={"param1": "value1"},
        )

        data = parsed.model_dump()
        assert data["command"] == "test"
        assert data["params"] == {"param1": "value1"}


class TestParamType:
    """测试参数类型"""

    def test_integer_parsing(self):
        """测试整数解析"""
        parser = CommandParser()
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="count", param_type=ParamType.INTEGER, required=True)
            ],
        )
        parser.register_command(command)

        _, params = parser.parse_args("test --count 42")
        assert params["count"] == 42
        assert isinstance(params["count"], int)

    def test_float_parsing(self):
        """测试浮点数解析"""
        parser = CommandParser()
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="ratio", param_type=ParamType.FLOAT, required=True)
            ],
        )
        parser.register_command(command)

        _, params = parser.parse_args("test --ratio 3.14")
        assert abs(params["ratio"] - 3.14) < 0.001
        assert isinstance(params["ratio"], float)

    def test_json_parsing(self):
        """测试JSON解析"""
        parser = CommandParser()
        command = CommandDefinition(
            name="test",
            params=[
                CommandParam(name="data", param_type=ParamType.JSON, required=True)
            ],
        )
        parser.register_command(command)

        _, params = parser.parse_args('test --data \'{"key": "value"}\'')
        assert params["data"]["key"] == "value"
