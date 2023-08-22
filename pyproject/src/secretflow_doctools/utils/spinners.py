from __future__ import annotations

import asyncio
import shutil
import sys
from collections import deque
from collections.abc import MutableSequence
from contextlib import contextmanager
from typing import (
    AsyncIterable,
    Generator,
    Iterable,
    List,
    Literal,
    Optional,
    Tuple,
    Union,
)

import asyncstdlib as astd
import tqdm
from loguru import logger
from termcolor import colored
from typing_extensions import TypeAlias

from .logging import no_spinner

BarConfig: TypeAlias = 'Tuple[Union[None, Literal["stdout"]], tqdm.tqdm]'


def dumb_textwrap(lines: Iterable[str], width=80) -> Iterable[str]:
    """
    Split an iterable of strings into strings of at most `width` characters.

    This is called "dumb" because it is dumber than :func:`textwrap.wrap`.
    """
    return (line[i : i + width] for line in lines for i in range(0, len(line), width))


@contextmanager
def progress_bar_with_text_stream(
    output: AsyncIterable[str],
    *config: BarConfig,
) -> Generator[Tuple[AsyncIterable[str], Tuple[tqdm.tqdm, ...]], None, None]:
    """Display a docker-like progress bar."""

    pbars: List[Tuple[bool, tqdm.tqdm]] = []
    """(managed, pbars)"""

    for bar_type, bar in config:
        if bar_type == "stdout":
            pbars.append((True, bar))
        else:
            pbars.append((False, bar))

    buffer = deque(maxlen=sum(managed for managed, _ in pbars))

    async def iterator():
        async for line in output:
            columns, _ = shutil.get_terminal_size()
            columns = max(columns - 2, 80)
            buffer.extend(dumb_textwrap([line], width=columns))
            managed_bars = [bar for managed, bar in pbars if managed]
            for idx, line in enumerate(buffer):
                managed_bars[idx].set_description(line.rstrip())
            yield line

    try:
        yield iterator(), tuple(bar for _, bar in pbars)
    finally:
        for _, bar in pbars:
            bar.close()


def iter_stream_reader(stream: asyncio.StreamReader):
    return astd.map(lambda line: line.decode("utf-8"), astd.iter(stream.readline, b""))


async def peek_subprocess_output(
    stream: asyncio.StreamReader,
    lines=4,
    *,
    spinner: tqdm.tqdm,
    refresh_interval=1,
    buffer: Optional[MutableSequence[str]] = None,
) -> None:
    bars: List[BarConfig] = [(None, spinner)]

    for _ in range(lines):
        bar = tqdm.tqdm(bar_format=colored("{desc}", attrs=["dark"]))
        bars.append(("stdout", bar))

    stream_iter = iter_stream_reader(stream)

    if no_spinner():
        async for line in stream_iter:
            if buffer is not None:
                buffer.append(line)
            logger.info(line.rstrip(), file=sys.stderr)

    else:
        pbar = progress_bar_with_text_stream(stream_iter, *bars)

        async def update_timer():
            while True:
                await asyncio.sleep(refresh_interval)
                spinner.update(1)

        async def collect_output():
            with pbar as (stdout, _):
                async for line in stdout:
                    if buffer is not None:
                        buffer.append(line)
                    spinner.update(1)

        timer = asyncio.create_task(update_timer())
        read_output = asyncio.create_task(collect_output())

        done, pending = await asyncio.wait(
            [timer, read_output],
            return_when=asyncio.FIRST_COMPLETED,
        )

        for task in pending:
            task.cancel()
