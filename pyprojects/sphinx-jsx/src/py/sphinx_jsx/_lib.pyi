from pathlib import Path
from typing import Optional, Protocol, Tuple, Union

from typing_extensions import TypeAlias

class WellKnownSymbols(Protocol):
    jsx: str
    jsxs: str
    fragment: str
    trans: str
    gettext: str
    url: str

class SphinxBundler:
    # this is actually __new__
    def __init__(self, symbols: WellKnownSymbols): ...
    def make_document(self, path: Path, source: str) -> SphinxDocument: ...
    def seal_document(self, document: SphinxDocument): ...

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
