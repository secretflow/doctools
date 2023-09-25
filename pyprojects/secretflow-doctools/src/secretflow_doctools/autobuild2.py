from __future__ import annotations

import asyncio
import sys
from contextlib import suppress
from itertools import chain
from pathlib import Path
from typing import Container, Iterable, Optional, Set, Tuple

import click
from loguru import logger
from pathspec import PathSpec
from pathspec.patterns.gitwildmatch import GitWildMatchPattern
from watchfiles import Change, awatch

from .workspace import sphinx_info_from_args

DOCUMENTATION_SOURCE_TYPES = (".rst", ".md", ".ipynb")


def gitignore_filter(*extra_patterns: str):
    ignores: str = "\n".join(extra_patterns)

    root = Path.cwd()

    for parent in chain([Path.cwd()], Path.cwd().parents):
        if parent.joinpath(".gitignore").exists():
            with open(parent.joinpath(".gitignore")) as f:
                ignores += f.read()
                ignores += "\n"
        if parent.joinpath(".git").is_dir():
            root = parent
            break

    spec = PathSpec.from_lines(GitWildMatchPattern, ignores.splitlines())

    def watch_filter(change: Change, path: str):
        try:
            pathname = str(Path(path).relative_to(root))
        except ValueError:
            pathname = Path(path).name
        return not spec.match_file(pathname)

    return watch_filter


async def autobuild2(
    sphinx_args: Iterable[str],
    *,
    watched_paths: Iterable[str],
    source_types: Container[str] = DOCUMENTATION_SOURCE_TYPES,
    ignored_paths: Iterable[str] = (),
    initial_build: bool = True,
    stop_event: Optional[asyncio.Event] = None,
):
    source_types = source_types or DOCUMENTATION_SOURCE_TYPES

    async def build_once(*changed_files: str):
        proc = await asyncio.create_subprocess_exec(
            sys.executable,
            "-m",
            "sphinx",
            *sphinx_args,
            *changed_files,
            stdout=None,
            stderr=None,
        )
        return await proc.wait()

    if initial_build:
        logger.info("Running initial build")
        yield await build_once()

    paths = [Path(p) for p in watched_paths]

    logger.info("Watching for file changes in:")
    for path in paths:
        logger.info(path.resolve())

    async for changes in awatch(
        *paths,
        watch_filter=gitignore_filter(*ignored_paths),
        debounce=500,
        stop_event=stop_event,
    ):
        changed_source_files: Set[str] = set()

        for change, path in changes:
            if (
                change == Change.modified
                or change == Change.added
                and Path(path).suffix in source_types
            ):
                changed_source_files.add(path)

        logger.info("Detected file changes, rebuilding ...")

        if changed_source_files:
            logger.info(f'Changed source files: {", ".join(changed_source_files)}')

        yield await build_once(*changed_source_files)


@click.command(
    name="autobuild2",
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.option("--source", multiple=True)
@click.option("--ignore", multiple=True)
@click.option("--initial-build/--no-initial-build", default=True)
@click.argument("sphinx-args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def autobuild2_command(
    ctx: click.Context,
    *,
    source: Tuple[str, ...] = (),
    ignore: Tuple[str, ...] = (),
    initial_build: bool = True,
    sphinx_args: Tuple[str, ...],
):
    info = sphinx_info_from_args(*sphinx_args, ctx=ctx)

    async def main():
        async for returncode in autobuild2(
            [*ctx.args, *sphinx_args],
            watched_paths=info.sourcedir,
            ignored_paths=ignore,
            source_types=source,
            initial_build=initial_build,
        ):
            if returncode != 0:
                logger.warning(f"Last build finished with status code {returncode}")
            else:
                logger.info("Build done")

    with suppress(KeyboardInterrupt):
        asyncio.run(main())


if __name__ == "__main__":
    autobuild2_command()
