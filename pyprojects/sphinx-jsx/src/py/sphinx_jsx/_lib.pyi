from pathlib import Path
from typing import Optional, Protocol

class SphinxOptions(Protocol):
    srcdir: Path
    confdir: Path
    outdir: Path

class SphinxBundler:
    def __new__(cls, options: SphinxOptions): ...
    def init(self) -> None: ...
    def chunk(
        self,
        component: str,
        attrs: str,
        *,
        file_name: Optional[str] = None,
        line_number: Optional[int] = None,
        raw_source: Optional[str] = None,
    ) -> None: ...
