from pathlib import Path

import click
from watchfiles import run_process
from watchfiles.filters import DefaultFilter
from watchfiles.main import Change


@click.group()
def maturin_utils():
    pass


class RustFilter(DefaultFilter):
    def __call__(self, change: Change, path: str) -> bool:
        return path.endswith(".rs") and super().__call__(change, path)


@maturin_utils.command()
def watch():
    run_process(
        Path.cwd() / "src",
        target="maturin develop --skip-install",
        watch_filter=RustFilter(),
    )
