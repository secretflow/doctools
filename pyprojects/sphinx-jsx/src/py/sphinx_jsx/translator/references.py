from pathlib import Path
from typing import List, Literal, Optional

from docutils import nodes
from sphinx import addnodes

from sphinx_jsx.syntax.jsx.models import JSON_NULL, JSFragment, JSXElement

from .base import BaseJSXTranslator
from .pseudo import Pseudo

ReferenceType = Literal["fragment", "internal", "external"]


class Link(JSXElement):
    href: str
    title: Optional[str] = None
    download: Optional[str] = None
    reftype: ReferenceType


class Footnote(JSXElement):
    backrefs: List[str] = []
    label: JSFragment = JSON_NULL

    class Reference(JSXElement):
        href: Optional[str] = None


class ReferenceMarkupTranslator(BaseJSXTranslator):
    def visit_reference(self, node: nodes.reference):
        self.enter_nesting(node, self.add_reference(node))

    def visit_footnote(self, node: nodes.footnote):
        elem = Footnote(backrefs=node.get("backrefs", []))
        self.enter_nesting(node, elem)

    def visit_footnote_reference(self, node: nodes.footnote_reference):
        reference = self.add_reference(node)
        self.enter_nesting(node, Footnote.Reference(href=reference.href))

    def visit_label(self, node: nodes.Element):
        self.enter_nesting(node, Pseudo._Label())

    def depart_footnote(self, node: nodes.footnote):
        elem = self.leave_nesting(node, Footnote)
        elem.label = elem.remove_fragment(Pseudo._Label)

    def visit_citation(self, node: nodes.citation):
        elem = Footnote(backrefs=node.get("backrefs", []))
        self.enter_nesting(node, elem)

    def depart_citation(self, node: nodes.citation):
        elem = self.leave_nesting(node, Footnote)
        elem.label = elem.remove_fragment(Pseudo._Label)

    def visit_target(self, node: nodes.Element):
        raise nodes.SkipNode

    def visit_download_reference(self, node: addnodes.download_reference):
        link = self.enter_nesting(node, self.add_reference(node))
        link.download = Path(link.href).name
