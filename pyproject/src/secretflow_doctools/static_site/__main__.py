import asyncio
import os
import shutil
from contextlib import contextmanager, suppress
from pathlib import Path

import click
from loguru import logger

from .runner import NodeProjectOptions, NodeProjectRunner


@click.group(hidden=True)
def cli():
    pass


@contextmanager
def error_handler_last_resort():
    try:
        yield
    except Exception as e:
        logger.opt(exception=os.getenv("DEBUG") == "true").error(e)
        logger.error("Error running command")
        exit(1)


@cli.command(hidden=True)
@click.argument("source", type=click.Path(exists=True, file_okay=False))
@click.argument("output", type=click.Path())
def build(source: str, output: str):
    async def main():
        runner = NodeProjectRunner(NodeProjectOptions())

        await runner.init()
        await runner.setup()

        logger.info("Preparing documentation sources")
        docs_source = Path(source).resolve()

        if docs_source != runner.docs_root:
            logger.info(f"Recreating {runner.docs_root}")
            shutil.rmtree(runner.docs_root, ignore_errors=True)
            shutil.copytree(docs_source, runner.docs_root)

        await runner.build()

        output_dir = Path(output).resolve()

        if output_dir != runner.build_dir:
            logger.info("Copying output")
            shutil.rmtree(output, ignore_errors=True)
            shutil.copytree(runner.build_dir, output, dirs_exist_ok=True)

        logger.success("Done!")

    with error_handler_last_resort():
        asyncio.run(main())


@cli.command(hidden=True)
@click.argument("source", type=click.Path(exists=True, file_okay=False))
def devserver(source: str):
    async def main():
        runner = NodeProjectRunner(NodeProjectOptions())

        await runner.init()
        await runner.setup()

        logger.info("Preparing documentation sources")
        docs_source = Path(source).resolve()

        if docs_source != runner.docs_root:
            logger.info("Symlinking source directory into workspace")

            if runner.docs_root.is_symlink():
                runner.docs_root.unlink()
            elif runner.docs_root.exists():
                raise ValueError(
                    f"{runner.docs_root} already exists and is not a symlink."
                    " Refusing to continue."
                )

            os.symlink(docs_source, runner.docs_root)

        await runner.devserver()

        logger.info("Shutting down the server")

    with error_handler_last_resort(), suppress(KeyboardInterrupt):
        asyncio.run(main())


if __name__ == "__main__":
    cli()
