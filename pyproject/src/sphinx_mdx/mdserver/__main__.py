from __future__ import annotations

import json
import signal
import sys
from typing import Optional

import click


@click.group()
@click.option("--server", help="Server origin", default=None)
@click.option("--script", help="Path to server script", default=None)
@click.pass_context
def cli(ctx: click.Context, server: Optional[str] = None, script: Optional[str] = None):
    ctx.ensure_object(dict)
    ctx.obj["server_origin"] = server
    ctx.obj["script_path"] = script


@cli.command()
@click.argument("input_ast", type=click.File("r"), default=sys.stdin)
@click.pass_context
def ast_to_md(ctx, input_ast):
    from .client import MarkdownClient

    with input_ast, MarkdownClient(**ctx.obj) as server:
        print(server.tree_to_markdown(json.load(input_ast)))


@cli.command()
@click.argument("input_doc", type=click.File("r"), default=sys.stdin)
@click.pass_context
def md_to_ast(ctx, input_doc):
    from .client import MarkdownClient

    with input_doc, MarkdownClient(**ctx.obj) as server:
        print(json.dumps(server.markdown_to_tree(input_doc.read()), indent=2))


@cli.command()
@click.argument("port", default="3000")
def serve(port):
    from .client import spawn_server

    signal.signal(signal.SIGINT, lambda *args: sys.exit(0))

    proc = spawn_server(port)
    print(f"Listening on {port}")
    proc.wait()


if __name__ == "__main__":
    cli()
