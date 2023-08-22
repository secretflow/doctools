from __future__ import annotations

import argparse
import asyncio
import os
import shlex
import shutil
import signal
import uuid
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Protocol, Tuple

import click
from loguru import logger

from .options import DocToolsOptions


class SphinxInfo(Protocol):
    sourcedir: str
    outputdir: str


def sphinx_info_from_args(
    *args: str,
    ctx: Optional[click.Context] = None,
) -> SphinxInfo:
    from sphinx.cmd.build import get_parser

    parser = get_parser()
    parser.exit_on_error = False

    try:
        return parser.parse_args(args)
    except argparse.ArgumentError as e:
        raise click.UsageError(str(e), ctx=ctx) from e


async def check_docker_prerequisites(
    image: str,
    update: bool = False,
    cli: str = "docker",
):
    logger.info("Checking Docker")

    proc = await asyncio.create_subprocess_exec(
        cli,
        "--version",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    if await proc.wait() != 0:
        raise RuntimeError("Docker is not installed. Abort.")
    stdout, stderr = await proc.communicate()
    logger.info(f"Using {stdout.decode().strip()}")

    logger.info("Checking image")
    proc = await asyncio.create_subprocess_exec(
        cli,
        "image",
        "inspect",
        image,
        stdout=asyncio.subprocess.DEVNULL,
        stderr=asyncio.subprocess.DEVNULL,
    )

    image_exists_locally = await proc.wait() == 0

    if not image_exists_locally or update:
        proc = await asyncio.create_subprocess_exec(
            cli,
            "pull",
            image,
            stdout=asyncio.subprocess.DEVNULL,
            stderr=asyncio.subprocess.DEVNULL,
        )
        imaged_pulled = await proc.wait() == 0
    else:
        imaged_pulled = False

    if not image_exists_locally and not imaged_pulled:
        raise RuntimeError(f"Image {image} is not available. Abort.")

    logger.info(f"Using image {image}")


class BuildWorkspace:
    MARKDOWN_SERVER_PORT = "8001"
    MARKDOWN_SERVER_ADDR = f"http://localhost:{MARKDOWN_SERVER_PORT}"

    def __init__(
        self,
        sphinx_args: Iterable[str],
        runner_options: DocToolsOptions,
    ):
        self.raw_args = tuple(sphinx_args)
        self.sphinx_options = sphinx_info_from_args(*self.raw_args)
        self.workspace_options = runner_options

        self.mdserver: Tuple[asyncio.subprocess.Process, str]

    @property
    def image_name(self):
        return self.workspace_options.DOCKER_IMAGE

    @property
    def source_root(self):
        return Path(self.sphinx_options.sourcedir)

    @property
    def output_root(self):
        return Path(self.sphinx_options.outputdir)

    @property
    def local_markdown_dir(self):
        return self.output_root.joinpath("mdx").resolve()

    @property
    def local_static_site_dir(self):
        return self.output_root.joinpath("html").resolve()

    @property
    def remote_markdown_dir(self):
        return "/workspace/build/mdx"

    @property
    def remote_static_site_dir(self):
        return "/workspace/build/html"

    @property
    def sphinx_build_args(self):
        args = [
            "-b",
            "mdx",
            "-D",
            f"mdx_mdserver_origin={self.MARKDOWN_SERVER_ADDR}",
            *self.raw_args,
        ]
        outputdir_idx = args.index(self.sphinx_options.outputdir)
        args[outputdir_idx] = str(self.local_markdown_dir.resolve())
        return tuple(args)

    def create_docker_command(
        self,
        *args: str,
        interactive: bool = True,
        container_name: Optional[str] = None,
        ports: Optional[Dict[str, str]] = None,
        extra_docker_args: Iterable[str] = (),
    ):
        bind_mounts = [
            "--mount",
            shlex.quote(
                f"type=bind,source={self.local_markdown_dir}"
                f",target={self.remote_markdown_dir}"
            ),
            "--mount",
            shlex.quote(
                f"type=bind,source={self.local_static_site_dir}"
                f",target={self.remote_static_site_dir}"
            ),
        ]

        port_mapping: List[str] = []
        ports = ports or {}

        for host_port, container_port in ports.items():
            port_mapping.append("-p")
            port_mapping.append(shlex.quote(f"{host_port}:{container_port}"))

        command = [
            self.workspace_options.DOCKER_CLI_PATH,
            "run",
            "-t",
        ]

        if interactive:
            command.append("-i")

        if container_name:
            command.append("--name")
            command.append(container_name)

        command.append("--rm")
        command.extend(bind_mounts)
        command.extend(port_mapping)
        command.extend(extra_docker_args)
        command.append(self.workspace_options.DOCKER_IMAGE)
        command.extend(args)

        return tuple(command)

    async def kill_container(self, name: str, sig: int = signal.SIGINT):
        return await asyncio.create_subprocess_exec(
            self.workspace_options.DOCKER_CLI_PATH,
            "kill",
            "-s",
            str(int(sig)),
            name,
            stdin=asyncio.subprocess.DEVNULL,
            stdout=asyncio.subprocess.DEVNULL,
            stderr=asyncio.subprocess.DEVNULL,
        )

    async def prerequisites(self):
        illegal_options = {"-b", "-M"} & set(self.raw_args)
        if illegal_options:
            raise ValueError(
                f"{','.join(illegal_options)} must not be specified for Sphinx args."
            )

        await check_docker_prerequisites(
            self.workspace_options.DOCKER_IMAGE,
            self.workspace_options.DOCKER_IMAGE_PULL_LATEST,
            self.workspace_options.DOCKER_CLI_PATH,
        )

        os.makedirs(self.local_markdown_dir, exist_ok=True)

        shutil.rmtree(self.local_static_site_dir, ignore_errors=True)
        os.makedirs(self.local_static_site_dir, exist_ok=True)

    @asynccontextmanager
    async def start(self):
        if getattr(self, "mdserver", None):
            raise RuntimeError("Already started.")

        # TODO: preferrably a context manager
        # Copilot says "preferrably docker-compose" and honestly true
        name = f"mdserver-{uuid.uuid4()}"
        command = self.create_docker_command(
            "serve",
            "markdown-server",
            "-p",
            self.MARKDOWN_SERVER_PORT,
            interactive=False,
            container_name=name,
            ports={self.MARKDOWN_SERVER_PORT: self.MARKDOWN_SERVER_PORT},
        )

        logger.info("Starting workspace")

        proc = await asyncio.create_subprocess_exec(
            *command,
            stdin=asyncio.subprocess.DEVNULL,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        self.mdserver = proc, name

        # FIXME: wait for server to start
        await proc.stdout.readline()

        try:
            yield
        finally:
            _, container_name = self.mdserver
            # FIXME: SIGKILL
            await self.kill_container(container_name, signal.SIGKILL)
            del self.mdserver
