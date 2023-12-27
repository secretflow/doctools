from typing import Dict, Optional

from sphinx_jsx.syntax import jsx


class NotebookCell(jsx.m.JSXElement):
    number: Optional[int] = None

    source: jsx.m.JSFragment = jsx.m.JSON_NULL
    output: jsx.m.JSFragment = jsx.m.JSON_NULL
    stdout: jsx.m.JSFragment = jsx.m.JSON_NULL
    stderr: jsx.m.JSFragment = jsx.m.JSON_NULL

    class _CodeArea(jsx.m.PseudoElement):
        prompt: Optional[str] = None
        stderr: bool = False

    class _RawText(jsx.m.PseudoElement):
        text: str

    class _Cell(jsx.m.JSXElement):
        prompt: str = ""
        fallback: Dict[str, str] = {}

    class Source(_Cell):
        pass

    class Output(_Cell):
        pass

    class StdOut(_Cell):
        pass

    class StdErr(_Cell):
        pass
