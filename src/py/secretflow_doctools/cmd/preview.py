import os
import subprocess
import sys
from contextlib import suppress
from typing import Optional

from pydantic_settings import BaseSettings

from secretflow_doctools.cmd.util import SphinxPaths
from secretflow_doctools.server import PreviewServerSettings
from secretflow_doctools.utils.subprocess import running


class PreviewServerArgs(BaseSettings):
    config_dir: Optional[str] = None
    source_dir: Optional[str] = None
    output_dir: Optional[str] = None

    model_config = PreviewServerSettings.model_config


def preview(
    *,
    flask_args: tuple[str, ...],
    **cli_args,
):
    import secretflow_doctools.server
    from secretflow_doctools.server import create_app

    args = PreviewServerArgs.model_validate({k: v for k, v in cli_args.items() if v})

    paths = SphinxPaths.check(
        config_dir=args.config_dir,
        source_dir=args.source_dir,
        output_dir=args.output_dir,
    )

    wsgi_app = f"{secretflow_doctools.server.__name__}:{create_app.__name__}()"

    settings = PreviewServerSettings(output_dir=paths.output_dir.joinpath("esm"))
    env_vars = {**os.environ, **settings.model_dump_env()}

    with suppress(KeyboardInterrupt):
        subprocess.run(
            running(
                sys.executable,
                "-m",
                "flask",
                "--app",
                wsgi_app,
                "run",
                *flask_args,
            ),
            env=env_vars,
            stdout=None,
            stderr=None,
            text=True,
        )
