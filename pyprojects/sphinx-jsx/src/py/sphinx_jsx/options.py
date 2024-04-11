from typing import List, Optional

from pydantic import BaseModel


class SphinxConfig(BaseModel):
    extensions: Optional[List[str]]
    myst_enable_extensions: Optional[List[str]]


class SphinxOptions(BaseModel):
    conf: SphinxConfig
    srcdir: str
    outdir: str
