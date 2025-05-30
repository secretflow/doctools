import shutil
import sysconfig
import tarfile
from importlib.resources import files
from pathlib import Path

from loguru import logger

from secretflow_doctools.l10n import gettext as _


def get_js_binary() -> tuple[str, list[str]]:
    import secretflow_doctools

    shared_scripts = Path(sysconfig.get_path("scripts"))

    bin_path = shared_scripts.joinpath("secretflow-doctools-js-cli")
    if bin_path.exists():
        return str(bin_path.resolve()), []

    bin_path = shared_scripts.joinpath("secretflow-doctools-js-cli.exe")
    if bin_path.exists():
        return str(bin_path.resolve()), []

    module_dir = files(secretflow_doctools)

    bin_path = Path(str(module_dir.joinpath("js/cli")))
    if bin_path.exists():
        return str(bin_path.resolve()), []

    bin_path = Path(str(module_dir.joinpath("js/cli.exe")))
    if bin_path.exists():
        return str(bin_path.resolve()), []

    script_path = Path(str(module_dir.joinpath("js/cli.js")))
    if not script_path.exists():
        raise fatal(_("a required file {file} was not found").format(file=script_path))

    if deno := shutil.which("deno"):
        return deno, ["--allow-all", str(script_path)]
    elif node := shutil.which("node"):
        return node, [str(script_path)]
    else:
        raise fatal(_("neither deno nor node is available on PATH"))


def get_js_static():
    import secretflow_doctools

    module_dir = files(secretflow_doctools)
    tar_path = Path(str(module_dir.joinpath("js/web.tar")))
    return tarfile.open(tar_path, "r")


def fatal(message: str):
    error = RuntimeError(message)
    logger.error(message)
    if any((p == "site-packages" for p in Path(__file__).parts)):
        hint = _(
            "help: the package `secretflow-doctools` was not installed correctly."
            " please reinstall it."
        )
        logger.critical(hint)
    else:
        hint = _(
            "in development: please run `pnpm nx build:sdist` to ensure"
            " JavaScript resources are compiled correctly"
        )
        logger.warning(hint)
    raise error
