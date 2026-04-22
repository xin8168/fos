"""
FOS CLI 主程序

提供命令行界面入口
"""

import asyncio
import json
from pathlib import Path
from typing import Any, Dict, Optional

import click
import httpx
import typer
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table

from fos_cli.models import (
    ApiResponse,
    EventQuery,
    EventType,
    FosCommand,
    FosEvent,
    SixAnchor,
)

# 创建 Typer 应用
app = typer.Typer(
    name="fos",
    help="FOS CLI - 确定性执行主板的明文交互入口",
    add_completion=False,
)

# 创建 Rich Console
console = Console()

# 默认配置
DEFAULT_GATEWAY_URL = "http://localhost:8080"
DEFAULT_TIMEOUT = 30.0


def get_gateway_url() -> str:
    """获取 Gateway URL"""
    import os

    return os.getenv("FOS_GATEWAY_URL", DEFAULT_GATEWAY_URL)


def get_timeout() -> float:
    """获取超时时间"""
    import os

    return float(os.getenv("FOS_TIMEOUT", DEFAULT_TIMEOUT))


async def send_command(command: FosCommand) -> ApiResponse:
    """发送命令到 Gateway"""
    url = f"{get_gateway_url()}/api/v1/command"

    async with httpx.AsyncClient(timeout=get_timeout()) as client:
        response = await client.post(
            url,
            json=command.model_dump(),
            headers={"Content-Type": "application/json"},
        )
        data = response.json()
        return ApiResponse(**data)


async def get_event(event_id: str) -> Optional[FosEvent]:
    """获取事件详情"""
    url = f"{get_gateway_url()}/api/v1/event/{event_id}"

    async with httpx.AsyncClient(timeout=get_timeout()) as client:
        response = await client.get(url)
        if response.status_code == 200:
            data = response.json()
            return FosEvent(**data)
        return None


async def query_events(query: EventQuery) -> list[FosEvent]:
    """查询事件列表"""
    url = f"{get_gateway_url()}/api/v1/events"

    async with httpx.AsyncClient(timeout=get_timeout()) as client:
        response = await client.get(url, params=query.model_dump())
        if response.status_code == 200:
            data = response.json()
            return [FosEvent(**item) for item in data.get("events", [])]
        return []


@app.command()
def run(
    event: str = typer.Option(..., "--事件", "-e", help="事件名称"),
    steps: str = typer.Option(..., "--步骤", "-s", help="执行步骤（逗号分隔）"),
    judgment_logic: str = typer.Option(..., "--判断逻辑", "-j", help="判断逻辑"),
    verification_standard: str = typer.Option(
        ..., "--校验标准", "-v", help="校验标准"
    ),
    location: str = typer.Option(..., "--地点", "-l", help="执行地点"),
    subject: str = typer.Option(..., "--主体", "-u", help="执行主体"),
) -> None:
    """执行 FOS 命令"""
    # 创建 6维锚定
    anchor = SixAnchor(
        event=event,
        steps=[s.strip() for s in steps.split("，")],
        judgment_logic=judgment_logic,
        verification_standard=verification_standard,
        location=location,
        subject=subject,
    )

    # 创建命令
    command = FosCommand(anchor=anchor)

    console.print("[bold blue]FOS 命令执行[/bold blue]")
    console.print(f"事件: {event}")
    console.print(f"步骤: {len(anchor.steps)} 个")

    # 发送命令
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("正在执行...", total=None)
        result = asyncio.run(send_command(command))
        progress.remove_task(task)

    if result.success:
        console.print("[bold green]✓ 命令已提交[/bold green]")
        console.print(f"事件ID: {result.data.get('event_id', 'N/A')}")
        console.print(f"状态: {result.data.get('status', 'N/A')}")
    else:
        console.print("[bold red]✗ 命令执行失败[/bold red]")
        console.print(f"错误: {result.message}")


@app.command()
def reuse(
    event_name: str = typer.Argument(..., help="要复用的事件名称"),
) -> None:
    """复用成功事件"""
    console.print(f"[bold blue]复用事件: {event_name}[/bold blue]")

    # TODO: 实现事件复用逻辑
    console.print("[yellow]功能开发中...[/yellow]")


@app.command()
def list_events(
    event_type: Optional[str] = typer.Option(None, "--type", "-t", help="事件类型"),
    limit: int = typer.Option(20, "--limit", "-n", help="显示数量"),
) -> None:
    """列出事件"""
    query = EventQuery(
        event_type=EventType(event_type) if event_type else None,
        limit=limit,
    )

    events = asyncio.run(query_events(query))

    table = Table(title="FOS 事件列表")
    table.add_column("ID", style="cyan")
    table.add_column("名称", style="green")
    table.add_column("类型", style="yellow")
    table.add_column("状态", style="blue")
    table.add_column("创建时间", style="magenta")

    for event in events:
        table.add_row(
            event.id[:8],
            event.name,
            event.event_type.value,
            event.status.value,
            str(event.created_at),
        )

    console.print(table)


@app.command()
def show(event_id: str = typer.Argument(..., help="事件ID")) -> None:
    """显示事件详情"""
    event = asyncio.run(get_event(event_id))

    if event is None:
        console.print("[bold red]事件不存在[/bold red]")
        return

    console.print(f"[bold blue]事件详情[/bold blue]")
    console.print(f"ID: {event.id}")
    console.print(f"名称: {event.name}")
    console.print(f"类型: {event.event_type.value}")
    console.print(f"状态: {event.status.value}")
    console.print(f"创建时间: {event.created_at}")

    console.print("\n[bold]6维锚定:[/bold]")
    console.print(f"  事件: {event.anchor.event}")
    console.print(f"  步骤: {', '.join(event.anchor.steps)}")
    console.print(f"  判断逻辑: {event.anchor.judgment_logic}")
    console.print(f"  校验标准: {event.anchor.verification_standard}")
    console.print(f"  地点: {event.anchor.location}")
    console.print(f"  主体: {event.anchor.subject}")

    if event.result:
        console.print("\n[bold]执行结果:[/bold]")
        console.print(f"  成功: {event.result.success}")
        console.print(f"  输出: {event.result.output}")
        if event.result.error:
            console.print(f"  错误: {event.result.error}")
        console.print(f"  执行时间: {event.result.duration_ms}ms")


@app.command()
def version() -> None:
    """显示版本信息"""
    from fos_cli import __version__

    console.print(f"[bold blue]FOS CLI[/bold blue] version {__version__}")


@app.command()
def status() -> None:
    """显示系统状态"""
    url = f"{get_gateway_url()}/api/v1/status"

    async def get_status() -> Optional[Dict[str, Any]]:
        async with httpx.AsyncClient(timeout=get_timeout()) as client:
            response = await client.get(url)
            if response.status_code == 200:
                return response.json()
            return None

    status_data = asyncio.run(get_status())

    if status_data is None:
        console.print("[bold red]无法连接到 Gateway[/bold red]")
        return

    console.print("[bold blue]FOS 系统状态[/bold blue]")
    console.print(f"总请求数: {status_data.get('total_requests', 0)}")
    console.print(f"成功请求: {status_data.get('success_requests', 0)}")
    console.print(f"失败请求: {status_data.get('failed_requests', 0)}")
    console.print(f"拦截请求: {status_data.get('blocked_requests', 0)}")


def cli() -> None:
    """CLI 入口"""
    app()


if __name__ == "__main__":
    cli()
