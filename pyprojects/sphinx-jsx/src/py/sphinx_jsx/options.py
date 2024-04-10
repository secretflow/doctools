from pydantic import BaseModel


class SphinxOptions(BaseModel):
    srcdir: str
    outdir: str
