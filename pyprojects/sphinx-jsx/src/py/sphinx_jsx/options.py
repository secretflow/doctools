from pathlib import Path

from pydantic import BaseModel


class WellKnownSymbols(BaseModel):
    jsx: str = "_jsx"
    jsxs: str = "_jsxs"
    fragment: str = "_Fragment"
    trans: str = "_Trans"
    gettext: str = "_gettext"
    url: str = "_url"


class BuildOptions(BaseModel):
    srcdir: Path
    outdir: Path
