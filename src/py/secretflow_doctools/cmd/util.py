from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Optional, TypeVar

from loguru import logger
from pydantic import ValidationError
from pydantic_settings import BaseSettings
from sphinx.config import Config
from sphinx.errors import ConfigError
from sphinx.util.tags import Tags

from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.options import ProjectConfig, parse_config
from secretflow_doctools.vcs import RemoteVCS, git_origin, guess_remote_vcs


@dataclass
class SphinxPaths:
    config_dir: Path
    source_dir: Path
    output_dir: Path

    @classmethod
    def check(
        cls,
        *,
        config_dir: Optional[Path | str],
        source_dir: Optional[Path | str],
        output_dir: Optional[Path | str],
    ):
        cwd = Path().cwd()

        def find_config_dir(config_dir: Optional[Path | str]):
            if config_dir:
                config_dir = cwd.joinpath(config_dir)
            else:
                config_dir = cwd
                if not config_dir.joinpath("conf.py").exists():
                    config_dir = cwd.joinpath("docs")
                if not config_dir.joinpath("conf.py").exists():
                    config_dir = cwd.joinpath("source")
                if not config_dir.joinpath("conf.py").exists():
                    config_dir = cwd.joinpath("docs").joinpath("source")
                if not config_dir.joinpath("conf.py").exists():
                    config_dir = None
            return config_dir

        config_dir = find_config_dir(config_dir)

        if source_dir:
            source_dir = cwd.joinpath(source_dir)
            if not config_dir:
                config_dir = find_config_dir(source_dir)
        elif config_dir:
            source_dir = config_dir

        if output_dir:
            output_dir = cwd.joinpath(output_dir)
            if not config_dir:
                config_dir = find_config_dir(output_dir.parent)
        elif config_dir:
            if config_dir.name == "source":
                output_dir = config_dir.parent.joinpath("build")
            else:
                output_dir = config_dir.joinpath("_build")

        if not config_dir:
            logger.error(
                _(
                    "could not detect Sphinx config directory: "
                    "none of the following exists: `./conf.py`, `./docs/conf.py`, `./docs/source/conf.py`"
                )
            )
            logger.error(
                _(
                    "help: specify Sphinx config directory using the `--config-dir` option"
                )
            )

        if not config_dir or not source_dir or not output_dir:
            raise SystemExit(1)

        return cls(
            config_dir=Path(config_dir),
            source_dir=Path(source_dir),
            output_dir=Path(output_dir),
        )


@dataclass
class SphinxPreconditions:
    paths: SphinxPaths
    args: tuple[str, ...]
    remote: Optional[RemoteVCS] = None
    config: Optional[Config] = None
    project: Optional[ProjectConfig] = None

    def check(self):
        exit_code = 0

        def non_fatal(msg: str):
            nonlocal exit_code
            logger.error(msg)
            exit_code = 1

        reserved_args = {"-b", "--builder", "-M", "-c", "--conf-dir"}
        reserved_args = [arg for arg in self.args if arg in reserved_args]
        if reserved_args:
            non_fatal(
                _("the following options cannot be specified: {}").format(reserved_args)
            )

        logger.info(_("checking Sphinx config"))

        with fatal_on_invalid_sphinx_conf():
            config = Config.read(self.paths.config_dir, tags=Tags())
            project_config = parse_config(config, ProjectConfig)

        if "secretflow_doctools" not in project_config.extensions:
            non_fatal(_('"secretflow_doctools" must be in `extensions` in conf.py'))

        origin = git_origin()
        if not origin:
            remote = None
        else:
            remote = guess_remote_vcs(origin)
        if not remote:
            non_fatal(
                _(
                    "failed to get or parse origin url from git config,"
                    " which is required to configure project name"
                )
            )

        if exit_code:
            raise SystemExit(exit_code)

        self.remote = remote
        self.config = config
        self.project = project_config

        return self


@contextmanager
def fatal_on_invalid_sphinx_conf():
    def onerror(error):
        logger.error(error)
        raise SystemExit(1)

    with logger.catch(
        (ConfigError, ValidationError),
        level="WARNING",
        message=_("failed to validate Sphinx conf.py"),
        onerror=onerror,
    ):
        yield


_Env = TypeVar("_Env", bound=BaseSettings)


def require_env_vars(cls: type[_Env]) -> _Env:
    def onerror(error):
        if isinstance(error, ValidationError):
            prefix = cls.model_config.get("env_prefix", "")
            for err in error.errors():
                if err["type"] == "missing":
                    logger.warning(
                        _("provide the environment variable {env}"),
                        env=repr(prefix + "_".join(map(str, err["loc"]))).upper(),
                    )
        logger.error(error)
        raise SystemExit(1)

    with logger.catch(
        ValidationError,
        level="WARNING",
        message=_("missing required environment variables"),
        onerror=onerror,
    ):
        return cls()
