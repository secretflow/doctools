from __future__ import annotations

from collections import defaultdict
from pathlib import Path
from typing import DefaultDict, List, Union

import click
from loguru import logger
from pydantic import BaseSettings
from tabulate import tabulate
from termcolor import colored


class SimplePyPIOptions(BaseSettings):
    class Config:
        env_prefix = "SIMPLE_PYPI_"

    ROOT_DIRECTORY: Path = Path()


def list_packages(root: Union[str, Path]):
    packages: DefaultDict[str, List[str]] = defaultdict(list)
    for artifact in Path(root).glob("*/*"):
        name, filename = artifact.parts[-2:]
        if {".whl", ".tar", ".zip"} & set(Path(filename).suffixes):
            packages[name].append(filename)
    return [[k, "\n".join(v)] for k, v in packages.items()]


@click.command()
@click.option("-p", "--port", type=int, default=8091)
def pypi(port: int):
    from ..http.serve import serve_directory_forever

    options = SimplePyPIOptions()

    packages = list_packages(options.ROOT_DIRECTORY)

    if not packages:
        logger.error(f"No packages found in {options.ROOT_DIRECTORY}")
        exit(1)

    def on_start(*args):
        logger.info(f"Running a PyPI server at http://localhost:{port}/")
        logger.info("")
        logger.info("Usage:")
        logger.info(
            colored(
                "  pip: pip install --extra-index-url"
                f" http://localhost:{port}/ [package]",
                attrs=["bold"],
            )
        )
        logger.info("  rye: see https://rye-up.com/guide/sources/")
        logger.info("")
        package_table = tabulate(packages, headers=["name", "formats"])
        logger.info(f"Packages:\n{package_table}")

    serve_directory_forever(options.ROOT_DIRECTORY, port, on_start)


if __name__ == "__main__":
    pypi()
if __name__ == "__main__":
    pypi()
