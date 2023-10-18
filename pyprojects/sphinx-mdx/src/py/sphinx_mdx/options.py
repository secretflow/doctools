from __future__ import annotations

from pathlib import Path
from typing import Callable, List, Literal, Optional, Union

from pydantic import BaseModel, Extra
from sphinx.application import Sphinx
from sphinx.builders import Builder


class ScaffoldingOptions(BaseModel):
    emits: Literal["mdx", "js"]
    output_dir: str = "."
    assets_dir: str = "./_assets"


class PackageJSON(BaseModel):
    class Config:
        extra = Extra.allow

    name: str
    version: str
    sphinx: ScaffoldingOptions
    files: List[str] = []


class BuildEnvironment(BaseModel):
    class Config:
        extra = Extra.allow

    # from builder
    srcdir: Path
    outdir: Path

    # from env
    doc2path: Callable[[str], str]

    mdx_scaffolding: Optional[Path] = None
    mdx_output_path_normalizer: Optional[Callable[[str], str]] = None

    mdx_npm_package_name: str = "%(name)s"
    mdx_npm_package_version: Union[Callable, str] = "%(version)s"


def parse_options(app: Sphinx, builder: Builder):
    """Parse the configuration options for the MDX builder."""
    return BuildEnvironment(
        **vars(builder),
        **vars(builder.env),
        **{c.name: c.value for c in app.config},
    )


def setup_config(app: Sphinx):
    for k, f in BuildEnvironment.__fields__.items():
        if k.startswith("mdx_"):
            app.add_config_value(k, f.default, True)
