from docutils import nodes
from sphinx.application import Sphinx

from sphinx_jsx.syntax import jsx
from sphinx_jsx.translator import BaseJSXTranslator
from sphinx_jsx.utils.sphinx import override_handlers


class Graphviz(jsx.m.JSXElement):
    code: str


def visit_graphviz(self: BaseJSXTranslator, node):
    self.append_child(node, Graphviz(code=node["code"]))
    raise nodes.SkipNode


def setup(app: Sphinx):
    try:
        from sphinx.ext.graphviz import graphviz
    except ImportError:
        return

    override_handlers(app, graphviz, visit_graphviz)
