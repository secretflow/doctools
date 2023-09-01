from __future__ import annotations

from pathlib import Path
from typing import Callable, Optional, Union

from pydantic import BaseModel, Extra
from sphinx.application import Sphinx
from sphinx.config import Config


class MDXOptions(BaseModel):
    class Config:
        extra = Extra.allow

    mdx_output_file_suffix: str = ".mdx"
    mdx_output_path_normalizer: Optional[Callable[[str], str]] = None
    mdx_assets_output_dir: Union[Path, str] = "_assets"

    mdx_mdserver_origin: Optional[str] = None


def parse_options(config: Config) -> MDXOptions:
    """Parse the configuration options for the MDX builder."""
    return MDXOptions(**{c.name: c.value for c in config})


def setup_config(app: Sphinx):
    for k, f in MDXOptions.__fields__.items():
        app.add_config_value(k, f.default, True)
