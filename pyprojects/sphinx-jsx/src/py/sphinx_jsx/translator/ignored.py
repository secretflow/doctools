from docutils import nodes


class IgnoredMarkupTranslator:
    def visit_comment(self, node: nodes.comment):
        raise nodes.SkipNode

    def visit_substitution_definition(self, node: nodes.substitution_definition):
        raise nodes.SkipNode

    def visit_toctree(self, node: nodes.Element):
        raise nodes.SkipNode

    def visit_index(self, node: nodes.Element):
        raise nodes.SkipNode
