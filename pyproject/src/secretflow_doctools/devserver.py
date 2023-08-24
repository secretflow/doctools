from __future__ import annotations

import asyncio
import signal
import uuid
from contextlib import suppress
from typing import Tuple

import click
from loguru import logger
from termcolor import colored

from .autobuild2 import autobuild2
from .options import DocToolsOptions
from .workspace import BuildWorkspace


async def devserver_main(*args: str, port: int) -> int | None:
    workspace = BuildWorkspace(args, DocToolsOptions())

    try:
        await workspace.prerequisites()
    except click.UsageError:
        raise
    except Exception as e:
        logger.error(str(e))
        return 1

    async with workspace.start():
        devserver_task: asyncio.Task[int | None] | None = None
        devserver_stop = asyncio.Event()

        returncode = 0

        async def run_devserver():
            name = f"devserver-{uuid.uuid4()}"
            command = workspace.create_docker_command(
                "static-site",
                "devserver",
                workspace.remote_markdown_dir,
                interactive=False,
                container_name=name,
                ports={f"{port}": "8000"},
            )
            proc = await asyncio.create_subprocess_exec(
                *command,
                stdin=asyncio.subprocess.DEVNULL,
                stdout=None,
                stderr=None,
            )
            try:
                return await proc.wait()
            finally:
                await workspace.kill_container(name, signal.SIGKILL)

        def start_devserver():
            nonlocal devserver_task
            devserver_task = asyncio.create_task(run_devserver())
            devserver_task.add_done_callback(lambda _: check_devserver())
            logger.info(colored("Starting devserver", "cyan", attrs=["bold"]))

        def check_devserver():
            if not devserver_task:
                start_devserver()
                return

            if not devserver_task.done() or devserver_task.cancelled():
                return

            else:
                devserver_stop.set()
                exit_code = devserver_task.result()
                if exit_code is not None and exit_code > 0 and exit_code < 128:
                    logger.error(f"devserver exited with code {exit_code}")

        async for returncode in autobuild2(
            workspace.sphinx_build_args,
            watched_paths=[str(workspace.source_root)],
            stop_event=devserver_stop,
        ):
            if returncode != 0:
                logger.warning(f"Last build finished with status code {returncode}")
                continue

            logger.info("Finished emitting intermediate files")
            logger.info("Please wait for devserver to recompile.")

            check_devserver()

        return returncode


@click.command(
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    }
)
@click.option("-p", "--port", default=8000)
@click.argument("sphinx-args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def devserver(ctx: click.Context, port: int, sphinx_args: Tuple[str, ...]):
    with suppress(KeyboardInterrupt):
        exit(asyncio.run(devserver_main(*ctx.args, *sphinx_args, port=port)))
