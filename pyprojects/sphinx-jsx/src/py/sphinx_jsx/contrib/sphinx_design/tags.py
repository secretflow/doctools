from typing import Optional

from sphinx_jsx.syntax import jsx


class SphinxDesign:
    class Card(jsx.m.JSXElement):
        href: Optional[str] = None
