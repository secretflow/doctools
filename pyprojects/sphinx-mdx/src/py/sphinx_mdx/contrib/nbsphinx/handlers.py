from docutils import nodes

from sphinx_mdx import mdx
from sphinx_mdx.translator import MDXTranslator

from .nodes import nbsphinx_container


def visit_nbsphinx_container(self: MDXTranslator, node: nbsphinx_container):
    classes = node.get("classes", [])

    if "nbcell" in classes:
        component = mdx.BlockTag.new("Notebook.Cell")
        self.enter_nesting(node, component)

        def visit_literal_block(node: nodes.literal_block):
            if "prompt" in node.get("classes", []):
                raise nodes.SkipNode
            return False

        self.contextualize(node, visit_literal_block)

        return

    if "nbinput" in classes:
        self.context_info["nb_cell_type"] = "input"
        return

    elif "nboutput" in classes:
        self.context_info["nb_cell_type"] = "output"
        return

    raise nodes.SkipNode


def visit_CodeAreaNode(self: MDXTranslator, node: nodes.Element):
    if node.get("stderr"):
        raise nodes.SkipNode
    self.enter_nesting(
        node,
        mdx.BlockTag.new(
            "Notebook.CodeArea",
            prompt=node["prompt"],
            stderr=node["stderr"],
            type=self.context_info.get("nb_cell_type"),
        ),
    )


def depart_CodeAreaNode(self: MDXTranslator, node: nodes.Element):
    self.leave_nesting(node)


def visit_FancyOutputNode(self: MDXTranslator, node: nodes.Element):
    self.enter_nesting(
        node,
        mdx.BlockTag.new(
            "Notebook.FancyOutput",
            prompt=node["prompt"],
            type=self.context_info.get("nb_cell_type"),
        ),
    )


def depart_FancyOutputNode(self: MDXTranslator, node: nodes.Element):
    self.leave_nesting(node)
