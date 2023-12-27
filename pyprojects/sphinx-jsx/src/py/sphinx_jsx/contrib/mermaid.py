from typing import Optional

from docutils.nodes import SkipNode
from sphinx.application import Sphinx

from sphinx_jsx.syntax import jsx
from sphinx_jsx.translator import BaseJSXTranslator
from sphinx_jsx.utils.sphinx import override_handlers


class Mermaid(jsx.m.JSXElement):
    code: str
    align: Optional[str] = None


def visit_mermaid(self: BaseJSXTranslator, node):
    self.append_child(
        node,
        Mermaid(code=node["code"], align=node.get("align")),
    )
    raise SkipNode


def setup(app: Sphinx):
    try:
        from sphinxcontrib.mermaid import mermaid
    except ImportError:
        return

    override_handlers(app, mermaid, visit_mermaid)
