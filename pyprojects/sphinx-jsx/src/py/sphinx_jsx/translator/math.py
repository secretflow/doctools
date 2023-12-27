from typing import Optional

from sphinx_jsx.syntax import math
from sphinx_jsx.syntax.jsx.models import JSXElement, PseudoElement

from .base import BaseJSXTranslator


class Math(PseudoElement):
    class Block(JSXElement):
        source: str
        number: Optional[str] = None

    class Inline(JSXElement):
        source: str


class MathMarkupTranslator(BaseJSXTranslator):
    def visit_math__rendered(self, node: math.e.math__rendered):
        self.enter_nesting(node, Math.Inline(source=node["source"]))

    def visit_math_block__rendered(self, node: math.e.math_block__rendered):
        self.enter_nesting(node, Math.Block(source=node["source"]))
