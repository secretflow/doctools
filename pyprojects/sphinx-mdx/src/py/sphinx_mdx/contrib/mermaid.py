from docutils.nodes import SkipNode
from sphinx.application import Sphinx

from sphinx_mdx import mdx
from sphinx_mdx.translator import MDXTranslator
from sphinx_mdx.utils.sphinx import override_handlers


def visit_mermaid(self: MDXTranslator, node):
    self.append_child(
        node,
        mdx.BlockTag.new("Mermaid", code=node["code"], align=node.get("align")),
    )
    raise SkipNode


def setup(app: Sphinx):
    from sphinxcontrib.mermaid import mermaid  # pyright: ignore[reportMissingImports]

    override_handlers(app, mermaid, visit_mermaid)
