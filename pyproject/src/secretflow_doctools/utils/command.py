from __future__ import annotations

from typing import Tuple, Union

import click


def iter_click_commands(
    cli: Union[click.Command, click.Group],
    path: Tuple[str, ...] = (),
):
    yield " ".join(path), cli
    if isinstance(cli, click.Group):
        for key, subcommand in cli.commands.items():
            subpath = path + (key,)
            yield from iter_click_commands(subcommand, subpath)
