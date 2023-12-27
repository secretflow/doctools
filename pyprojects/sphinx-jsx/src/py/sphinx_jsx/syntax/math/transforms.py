from docutils import nodes
from sphinx.transforms.post_transforms import SphinxPostTransform

from sphinx_jsx._lib import math_to_html
from sphinx_jsx.syntax import html, rst

from .elements import math__rendered, math_block__rendered


class MathRenderingTransform(SphinxPostTransform):
    default_priority = 999
    builders = ("jsx",)

    def run(self, **kwargs):
        for node in self.document.findall(nodes.math):
            source = node.astext().strip()
            try:
                rendered = math_to_html(source)
            except ValueError:
                continue
            raw = nodes.raw("", rendered, format="html")
            tree = html.transforms.html_block_to_doctree(raw)
            math = math__rendered(node.rawsource, "", tree)
            math["source"] = source
            rst.transforms.replace_self(node, math)

        for node in self.document.findall(nodes.math_block):
            source = node.astext().strip()
            try:
                rendered = math_to_html(source, mode="block")
            except ValueError:
                continue
            raw = nodes.raw("", rendered, format="html")
            tree = html.transforms.html_block_to_doctree(raw)
            math = math_block__rendered(node.rawsource, tree)
            math["source"] = source
            rst.transforms.replace_self(node, math)
