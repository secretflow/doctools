from __future__ import annotations

import asyncio
import json
import os
from collections import deque
from pathlib import Path
from typing import List, Literal, Optional

import aiofiles
import tqdm
from loguru import logger
from pydantic import AnyHttpUrl, BaseSettings, ValidationError
from termcolor import colored

from ..utils.spinners import iter_stream_reader, peek_subprocess_output
from .options import PackageJSON


class SubprocessError(Exception):
    def __init__(self, command: str, exit_code: int, buffer: List[str]) -> None:
        super().__init__()
        self.command = command
        self.exit_code = exit_code
        self.buffer = buffer

    def __str__(self) -> str:
        lines = [
            f"Subprocess error: Command {self.command} exited with code"
            f" {self.exit_code}. Last 300 lines of output:\n",
            *self.buffer,
        ]
        return "".join(lines)


class NodeProjectOptions(BaseSettings):
    class Config:
        env_prefix = "STATIC_SITE_"

    WORKSPACE_ROOT: Path = Path()

    NODE_PREFIX: Optional[Path] = None
    NPM_REGISTRY: AnyHttpUrl = AnyHttpUrl("https://registry.npmjs.org/")

    INSTALL_DEPS: bool = True


class NodeProjectRunner:
    def __init__(self, options: Optional[NodeProjectOptions] = None):
        self.options = options or NodeProjectOptions()
        self.package: PackageJSON

    @property
    def cwd(self):
        return self.options.WORKSPACE_ROOT

    @property
    def docs_root(self):
        return self.cwd / self.package.sphinx_theme.docs_root

    @property
    def build_dir(self):
        return self.cwd / self.package.sphinx_theme.build_dir

    def _environ(self, *, ci: Literal["true", "false"] = "true"):
        environ = {**os.environ, "CI": ci}
        if self.options.NODE_PREFIX:
            bin = str((self.options.NODE_PREFIX / "bin").resolve())
            environ["PATH"] = f"{bin}:{environ['PATH']}"
        return environ

    async def run_npm_task_until_complete(self, *args: str, raise_on_error=True):
        proc = await asyncio.create_subprocess_exec(
            "npm",
            *args,
            cwd=self.cwd,
            env=self._environ(ci="true"),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )

        program = f"npm {' '.join(args)}"
        command = colored(program, attrs=["underline"])
        timer_fmt = colored(f"Running {command} (elapsed time: {{elapsed}})", "blue")
        timer = tqdm.tqdm(bar_format=timer_fmt)
        buffer: deque[str] = deque(maxlen=300)

        await peek_subprocess_output(proc.stdout, spinner=timer, buffer=buffer)

        try:
            await asyncio.wait_for(proc.wait(), timeout=30)
        except asyncio.TimeoutError:
            logger.warning(f"Timed out waiting for {command} to finish")
            proc.kill()

        exit_code = await proc.wait()

        if raise_on_error and exit_code > 0:
            raise SubprocessError(program, exit_code, list(buffer))

        return exit_code, buffer

    async def run_npm_task_forever(self, *args: str):
        proc = await asyncio.create_subprocess_exec(
            "npm",
            *args,
            cwd=self.cwd,
            env=self._environ(ci="false"),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )

        buffer: deque[str] = deque(maxlen=300)

        async for line in iter_stream_reader(proc.stdout):
            logger.info(line.rstrip())
            buffer.append(line)

        return await proc.wait(), buffer

    async def init(self):
        logger.info("Preparing theme")
        logger.info("Reading package.json")
        try:
            async with aiofiles.open(self.cwd / "package.json", "r") as f:
                package_json = json.loads(await f.read())
            self.package = PackageJSON(**package_json)
        except OSError:
            raise ValueError(
                f"Could not read package.json in {self.cwd}."
                " This directory cannot be used as a theme."
            ) from None
        except ValidationError as e:
            raise ValueError(
                f"package.json in {self.cwd} is not valid."
                " This directory cannot be used as a theme."
                f"\n===\n{e}"
            ) from e
        else:
            logger.info(f"Using theme: {self.package.name}@{self.package.version}")
            logger.info(f"Using NPM registry: {self.options.NPM_REGISTRY}")
            logger.info(f"Installation required: {self.options.INSTALL_DEPS}")

    async def setup(self):
        if self.options.INSTALL_DEPS:
            logger.info("Installing dependencies")
            await self.run_npm_task_until_complete(
                "install",
                "--loglevel",
                "info",
                "--registry",
                str(self.options.NPM_REGISTRY),
            )
        if self.package.scripts.setup:
            logger.info("Running setup script")
            await self.run_npm_task_until_complete("run", "scaffolding:setup")

    async def build(self):
        logger.info("Building static site")
        await self.run_npm_task_until_complete("run", "scaffolding:build")

    async def devserver(self):
        logger.info("Starting the development server")
        exit_code, output = await self.run_npm_task_forever("run", "scaffolding:start")
        # https://docs.python.org/3/library/subprocess.html#subprocess.CompletedProcess.returncode
        # > A negative value -N indicates that the child was terminated by signal N
        # (POSIX only).
        if exit_code == 0:
            return
        logger.error(f"Development server exited with code {exit_code}")
