import click
from dotenv import load_dotenv

from . import build, devserver, help, serve, static_site
from .utils.logging import configure_logging


class RootCommand(click.Group):
    def format_help_text(
        self,
        ctx: click.Context,
        formatter: click.HelpFormatter,
    ) -> None:
        from .help.__main__ import _print_all_commands

        subcommands = _print_all_commands(ctx)

        formatter.write_paragraph()
        formatter.write_text("Commands:")
        formatter.write(subcommands)
        formatter.write_paragraph()


@click.group(name="secretflow-doctools", hidden=True, cls=RootCommand)
@click.option(
    "-e",
    "--env-file",
    type=click.Path(dir_okay=False),
    default=".env",
)
def cli(env_file: str):
    load_dotenv(env_file)
    configure_logging()


cli.add_command(build.build, "build")
cli.add_command(devserver.devserver, "devserver")

cli.add_command(serve.cli, "serve")
cli.add_command(help.cli, "help")

cli.add_command(static_site.cli, "static-site")


if __name__ == "__main__":
    cli()
