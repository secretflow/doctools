import shlex
import subprocess
from contextlib import contextmanager

from loguru import logger

from secretflow_doctools.l10n import gettext as _


def running(*args) -> tuple[str, ...]:
    args = tuple(str(a) for a in args)
    logger.debug("running {cmd}", cmd=shlex.join(args))
    return args


@contextmanager
def fatal_on_subprocess_error(*args):
    cmd = running(*args)

    def onerror(*args, **kwargs):
        logger.critical(
            _("failed to run command, see above for errors\nCommand: {cmd}"),
            cmd=shlex.join(cmd),
        )
        raise SystemExit(1)

    with logger.catch(
        (OSError, subprocess.SubprocessError),
        level="WARNING",
        onerror=onerror,
    ):
        yield cmd
