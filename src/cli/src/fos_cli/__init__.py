"""
FOS CLI - 明文交互入口

FOS 的唯一人机交互层，提供中文明文指令输入
"""

__version__ = "0.1.0"
__author__ = "FOS Team"

from fos_cli.main import cli
from fos_cli.models import FosCommand, FosEvent, SixAnchor, FourElement
from fos_cli.parser import (
    ParsedCommand,
    CommandLineParser,
    CommandDefinition,
    CommandParam,
    CommandParser,
    ParamType,
    default_parser,
)

__all__ = [
    # Main CLI
    "cli",
    # Models
    "FosCommand",
    "FosEvent",
    "SixAnchor",
    "FourElement",
    # Parser
    "ParsedCommand",
    "CommandLineParser",
    "CommandDefinition",
    "CommandParam",
    "CommandParser",
    "ParamType",
    "default_parser",
]
