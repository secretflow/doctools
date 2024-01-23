from pathlib import Path
from typing import Optional, Protocol, Tuple, Union

from typing_extensions import TypeAlias

# @property due to https://peps.python.org/pep-0544/#covariant-subtyping-of-mutable-attributes
class WellKnownSymbols(Protocol):
    @property
    def jsx(self) -> str: ...
    @property
    def jsxs(self) -> str: ...
    @property
    def fragment(self) -> str: ...
    @property
    def trans(self) -> str: ...
    @property
    def gettext(self) -> str: ...
    @property
    def url(self) -> str: ...

class BuildOptions(Protocol):
    @property
    def srcdir(self) -> Path: ...
    @property
    def outdir(self) -> Path: ...

class SphinxBundler:
    # this is actually __new__
    def __init__(self, symbols: WellKnownSymbols): ...
    def make_document(self, path: Path, source: str) -> SphinxDocument: ...
    def seal_document(self, document: SphinxDocument): ...
    def build(self, options: BuildOptions) -> None: ...

SourcePositionLines: TypeAlias = Tuple[int, int]
SourcePositionRange: TypeAlias = Tuple[int, int, int, int]
SourcePosition = Union[SourcePositionLines, SourcePositionRange]

class SphinxDocument:
    def element(
        self,
        name: str,
        props: Optional[str] = None,
        *,
        position: Optional[SourcePosition] = None,
    ): ...
    def enter(self): ...
    def text(self, text: str): ...
    def exit(self): ...

class testing:
    @staticmethod
    def ast_string_to_ecma(expr: str) -> str: ...
