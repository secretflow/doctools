from sphinx_jsx.translator import BaseJSXTranslator
from sphinx_jsx.translator.references import Link

from .tags import SphinxDesign


def visit(self: BaseJSXTranslator, node):
    self.enter_nesting(node, SphinxDesign.Card())


def depart(self: BaseJSXTranslator, node):
    elem = self.leave_nesting(node, SphinxDesign.Card)
    try:
        anchor = elem.remove_child(Link)
    except ValueError:
        return
    elem.href = anchor.href
