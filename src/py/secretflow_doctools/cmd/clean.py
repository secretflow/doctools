import shutil
from typing import Optional

import click
from loguru import logger

from secretflow_doctools.cmd.util import SphinxPaths
from secretflow_doctools.l10n import gettext as _


def clean(
    config_dir: Optional[str],
    source_dir: Optional[str],
    output_dir: Optional[str],
):
    paths = SphinxPaths.check(
        config_dir=config_dir,
        source_dir=source_dir,
        output_dir=output_dir,
    )

    del config_dir
    del source_dir
    del output_dir

    logger.warning(_(f"will remove {paths.output_dir}"))

    if click.confirm(_("do you want to continue?")):
        shutil.rmtree(paths.output_dir, True)
