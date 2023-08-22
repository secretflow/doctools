from contextlib import suppress

import click
from loguru import logger

from . import http, pypi


@click.group(hidden=True)
def cli():
    pass


@cli.command(hidden=True)
@click.option("-p", "--port", default="3000")
def markdown_server(port: str):
    from sphinx_mdx.mdserver.client import spawn_server

    proc = spawn_server(port)
    logger.info(f"Markdown server running at http://localhost:{port}")

    with suppress(KeyboardInterrupt):
        proc.wait()


cli.add_command(http.cli, "http")
cli.add_command(pypi.cli, "pypi")


if __name__ == "__main__":
    cli()
