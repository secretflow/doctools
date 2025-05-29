from __future__ import annotations

from pathlib import Path
from typing import Callable, Optional, TypeVar, Union

from pydantic import BaseModel, ConfigDict, Field
from sphinx.application import Sphinx
from sphinx.config import Config

T = TypeVar("T", bound=BaseModel)


class ProjectConfig(BaseModel):
    extensions: list[str] = []


class GettextConfig(BaseModel):
    language: str
    locale_dirs: list[Path] = Field(min_length=1)


class MdxConfig(BaseModel):
    mdx_output_file_suffix: str = ".mdx"
    mdx_output_path_normalizer: Optional[Callable[[str], str]] = None
    mdx_assets_output_dir: Union[Path, str] = "_assets"

    model_config = ConfigDict(extra="ignore")


def setup_config(app: Sphinx, schema: type[T]):
    for k, f in schema.model_fields.items():
        app.add_config_value(k, f.default, "env")


def parse_config(config: Config, schema: type[T]) -> T:
    return schema.model_validate({**vars(config), **{c.name: c.value for c in config}})
