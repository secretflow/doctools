from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Extra, Field


class ThemeProperties(BaseModel):
    class Config:
        extra = Extra.ignore

    docs_root: str = "docs"
    build_dir: str = "dist"


class PackageScripts(BaseModel):
    class Config:
        extra = Extra.allow

    setup: Optional[str] = None
    start: Optional[str] = None
    build: Optional[str] = None


class PackageJSON(BaseModel):
    class Config:
        extra = Extra.allow

    sphinx_theme: ThemeProperties = Field(default_factory=ThemeProperties)

    name: str
    version: str
    scripts: PackageScripts = Field(default_factory=PackageScripts)
