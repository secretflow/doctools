import subprocess
import sys
from pathlib import Path
from typing import Optional

from loguru import logger

from secretflow_doctools.cmd.util import (
    SphinxPaths,
    SphinxPreconditions,
    fatal_on_invalid_sphinx_conf,
)
from secretflow_doctools.js.cli import get_js_binary
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.options import GettextConfig, parse_config
from secretflow_doctools.utils.subprocess import fatal_on_subprocess_error
from secretflow_doctools.vcs import (
    HeadRef,
    MainRef,
    NearestRef,
    PreciseRef,
    git_describe,
)


def build(
    config_dir: Optional[str],
    source_dir: Optional[str],
    output_dir: Optional[str],
    lang: tuple[str, ...],
    mapped_path: tuple[str, ...],
    sphinx_args: tuple[str, ...] = (),
):
    paths = SphinxPaths.check(
        config_dir=config_dir,
        source_dir=source_dir,
        output_dir=output_dir,
    )

    del config_dir
    del source_dir
    del output_dir

    options = SphinxPreconditions(paths=paths, args=sphinx_args).check()
    assert options.remote
    assert options.config

    repo = options.remote.name
    ref = "main"

    with logger.catch(
        ValueError,
        message=_("failed to get current tag, falling back to `main`"),
        level="WARNING",
    ):
        rev = git_describe()
        ref = rev.ref
        match rev.src:
            case PreciseRef():
                logger.info(
                    _("using {ref} as version, source: current commit is tagged"),
                    ref=repr(ref),
                )
            case NearestRef(branch=branch):
                logger.info(
                    _(
                        "using {ref} as version, source:"
                        " nearest tag on release branch {branch}"
                    ),
                    ref=repr(ref),
                    branch=repr(branch),
                )
            case MainRef() | HeadRef():
                ref = ref.replace("/", "-")
                logger.info(
                    _("using {ref} as version, source: current branch name"),
                    ref=repr(ref),
                )

    output_dir_mdx = paths.output_dir.joinpath("mdx")
    cwd = Path.cwd().resolve()

    logger.info(_("source dir: {dir}"), dir=paths.source_dir.relative_to(cwd))
    logger.info(_("output dir: {dir}"), dir=output_dir_mdx.relative_to(cwd))

    def build_one(lang: str):
        output_lang = lang.replace("_", "-")
        sphinx_lang = lang.replace("-", "_")

        with fatal_on_subprocess_error(
            sys.executable,
            "-m",
            "sphinx",
            "-b",
            "mdx",
            "-D",
            f"language={sphinx_lang}",
            "-c",
            paths.config_dir,
            paths.source_dir,
            output_dir_mdx.joinpath(repo).joinpath(ref).joinpath(output_lang),
            *options.args,
        ) as cmd:
            subprocess.run(cmd, stdout=None, stderr=None, text=True).check_returncode()

    if not lang:
        with fatal_on_invalid_sphinx_conf():
            gettext = parse_config(options.config, GettextConfig)
        lang = (gettext.language,)

    for lang_id in lang:
        build_one(lang_id)

    logger.info(_("compiling to JavaScript"))

    cmd, args = get_js_binary()

    args = [
        *args,
        "bundle",
        "-i",
        paths.output_dir.joinpath("mdx"),
        "-o",
        paths.output_dir.joinpath("esm"),
    ]

    for mapped in mapped_path:
        args.append("--redirect")
        args.append(mapped)

    with fatal_on_subprocess_error(cmd, *args) as cmd:
        subprocess.run(cmd, stdout=None, stderr=None, text=True).check_returncode()

    logger.success(
        _("to preview, run: {prog_name} preview -c {config_dir}"),
        prog_name=prog_name(),
        config_dir=paths.config_dir.relative_to(cwd),
    )


def prog_name():
    if sys.argv[0] == __file__:
        return "python -m secretflow_doctools"
    else:
        return "secretflow-doctools"
