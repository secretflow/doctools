from pathlib import Path

import click
from loguru import logger
from termcolor import colored


@click.command()
@click.option("-p", "--port", type=int, default=8000)
@click.argument(
    "root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
)
def http(root: str, port: int):
    from .serve import serve_directory_forever

    def on_start(*args):
        logger.info(f"Serving directory {root}")
        address = f"http://localhost:{port}/"
        logger.success(f"Open {colored(address, attrs=['underline'])} in your browser")

    serve_directory_forever(Path(root), port, on_start)


if __name__ == "__main__":
    http()
