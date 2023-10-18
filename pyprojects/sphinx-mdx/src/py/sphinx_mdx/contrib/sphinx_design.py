from docutils.nodes import SkipNode, container, reference
from sphinx.application import Sphinx

from sphinx_mdx import mdx
from sphinx_mdx.translator import MDXTranslator
from sphinx_mdx.utils.sphinx import SkipHandler, override_handlers


def visit_container(self: MDXTranslator, node):
    component_type = node.get("design_component")
    if not component_type:
        raise SkipHandler

    component = mdx.BlockTag.new("SphinxDesign", type=component_type)
    self.enter_nesting(node, component)

    if component_type == "card":

        def visit_reference(node: reference):
            if "sd-stretched-link" not in node["classes"]:
                return False
            component.attributes["href"] = node["refuri"]
            raise SkipNode

        def depart_reference(node: reference):
            pass

        self.contextualize(node, visit_reference, depart_reference)


def depart_container(self: MDXTranslator, node):
    self.leave_nesting(node)


def setup(app: Sphinx):
    override_handlers(app, container, visit_container, depart_container)
