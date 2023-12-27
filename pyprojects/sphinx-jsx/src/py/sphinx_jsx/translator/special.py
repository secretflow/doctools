from typing import Optional

from docutils import nodes

from sphinx_jsx.syntax.jsx.models import JSXElement, PseudoElement

from .base import BaseJSXTranslator


class Special(PseudoElement):
    class Problematic(JSXElement):
        reason: Optional[str] = None

    class Raw(JSXElement):
        format: str
        content: str


class SpecialMarkupTranslator(BaseJSXTranslator):
    def visit_problematic(self, node: nodes.problematic):
        self.enter_nesting(node, Special.Problematic())

    def visit_raw(self, node: nodes.raw):
        elem = Special.Raw(format=node["format"], content=node.astext())
        self.append_child(node, elem)
        raise nodes.SkipNode
