from docutils.nodes import SkipNode
from sphinx.application import Sphinx

from sphinx_mdx import mdx
from sphinx_mdx.translator import MDXTranslator
from sphinx_mdx.utils.sphinx import override_handlers


def visit_graphviz(self: MDXTranslator, node):
    self.append_child(node, mdx.BlockTag.new("Graphviz", code=node["code"]))
    raise SkipNode


def setup(app: Sphinx):
    from sphinx.ext.graphviz import graphviz

    override_handlers(app, graphviz, visit_graphviz)
