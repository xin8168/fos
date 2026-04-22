"""
FOS CLI 配置
"""

import os
from pathlib import Path
from typing import Optional

from pydantic import BaseModel
import yaml


class GatewayConfig(BaseModel):
    """Gateway 配置"""

    url: str = "http://localhost:8080"
    timeout: float = 30.0
    api_key: Optional[str] = None


class CLIConfig(BaseModel):
    """CLI 配置"""

    gateway: GatewayConfig = GatewayConfig()

    class Config:
        """Pydantic 配置"""

        env_prefix = "FOS_"


def load_config(config_path: Optional[Path] = None) -> CLIConfig:
    """加载配置文件"""
    # 默认配置路径
    if config_path is None:
        config_path = Path.home() / ".fos" / "config.yaml"

    if config_path.exists():
        with open(config_path, encoding="utf-8") as f:
            data = yaml.safe_load(f) or {}
            return CLIConfig(**data)

    return CLIConfig()


def save_config(config: CLIConfig, config_path: Optional[Path] = None) -> None:
    """保存配置文件"""
    if config_path is None:
        config_path = Path.home() / ".fos" / "config.yaml"

    config_path.parent.mkdir(parents=True, exist_ok=True)

    with open(config_path, "w", encoding="utf-8") as f:
        yaml.dump(config.model_dump(), f, default_flow_style=False)


def get_config() -> CLIConfig:
    """获取配置（优先使用环境变量）"""
    config = load_config()

    # 环境变量覆盖
    if url := os.getenv("FOS_GATEWAY_URL"):
        config.gateway.url = url
    if timeout := os.getenv("FOS_TIMEOUT"):
        config.gateway.timeout = float(timeout)
    if api_key := os.getenv("FOS_API_KEY"):
        config.gateway.api_key = api_key

    return config
