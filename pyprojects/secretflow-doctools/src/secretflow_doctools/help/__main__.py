import importlib
from typing import List, Set, Tuple, Type

import click
from pydantic import BaseSettings
from tabulate import tabulate
from termcolor import colored

from ..utils.command import iter_click_commands


def _print_all_commands(ctx: click.Context):
    root_command = ctx.find_root().command

    lines: List[Tuple[str, str]] = []

    for path, cmd in iter_click_commands(root_command):
        if not cmd.callback or cmd.hidden:
            continue
        lines.append((colored(path, "green"), cmd.short_help or cmd.help or ""))

    return tabulate(lines, tablefmt="plain", maxcolwidths=[40, 80])


@click.group(hidden=True)
def cli():
    pass


@cli.command()
@click.pass_context
def list_settings(ctx: click.Context):
    root_command = ctx.find_root().command
    printed: Set[Type[BaseSettings]] = set()

    lines: List[Tuple[str, str]] = []

    for _, cmd in iter_click_commands(root_command):
        if not cmd.callback:
            continue

        main = importlib.import_module(cmd.callback.__module__)

        for v in vars(main).values():
            if not (isinstance(v, type) and issubclass(v, BaseSettings)):
                continue
            if v in printed:
                continue

            printed.add(v)

            prefix = getattr(v.Config, "env_prefix", "")
            for name, field in v.__fields__.items():
                env_key = f"{prefix}{name}".upper()
                if isinstance(field.annotation, type):
                    annotation = field.annotation.__name__
                else:
                    annotation = str(field.annotation)

                lines.append(
                    (
                        colored(env_key, attrs=["bold"]),
                        f'{field.field_info.description or ""}'
                        f'\n{colored(annotation, "yellow")}',
                    )
                )

    print(tabulate(lines, tablefmt="plain"))


@cli.command()
@click.pass_context
def list_commands(ctx: click.Context):
    print(_print_all_commands(ctx))


if __name__ == "__main__":
    cli()
