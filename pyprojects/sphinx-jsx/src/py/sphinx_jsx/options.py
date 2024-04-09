from pathlib import Path

from pydantic import BaseModel


class SphinxOptions(BaseModel):
    srcdir: Path
    confdir: Path
    outdir: Path
