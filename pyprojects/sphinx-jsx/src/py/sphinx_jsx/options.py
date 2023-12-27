from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel, Extra
from sphinx.application import Sphinx
from sphinx.builders import Builder


class BuildOptions(BaseModel):
    class Config:
        extra = Extra.allow

    # from builder
    srcdir: Path
    outdir: Path


def parse_options(app: Sphinx, builder: Builder):
    """Parse the configuration options for the JSX builder."""
    return BuildOptions(
        srcdir=Path(builder.srcdir),
        outdir=Path(builder.outdir),
        **{c.name: c.value for c in app.config},
    )


def setup_config(app: Sphinx):
    for k, f in BuildOptions.__fields__.items():
        if k.startswith("jsx_"):
            app.add_config_value(k, f.default, True)
