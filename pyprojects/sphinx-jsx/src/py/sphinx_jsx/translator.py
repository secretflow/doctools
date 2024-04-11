from typing import Dict, NamedTuple

import orjson
from docutils import nodes
from sphinx.builders import Builder
from sphinx.util.docutils import SphinxTranslator


class Range(NamedTuple):
    start_line: int
    start_column: int
    end_line: int
    end_column: int


class SphinxJSXTranslator(SphinxTranslator):
    def __init__(
        self,
        document: nodes.document,
        builder: Builder,
        docname: str,
    ) -> None:
        from .builder import SphinxJSXBuilder

        if not isinstance(builder, SphinxJSXBuilder):
            raise TypeError("builder must be a SphinxJSXBuilder")

        super().__init__(document, builder)

        self.builder = builder
        self.docname = docname
        self.doctree = self.builder.source_map.open(self.builder.env.doc2path(docname))

    def visit_Element(self, node: nodes.Element):
        component = type(node).__name__

        file = str(node.source) if node.source else None
        line = node.line or None
        source = str(node.rawsource) if node.rawsource else None

        attrs = attrs_to_str(node.attributes)

        self.doctree.element(component, attrs, file=file, line=line, source=source)
        self.doctree.enter()

    def depart_Element(self, node: nodes.Element):
        self.doctree.exit()

    def visit_Text(self, node: nodes.Text):
        self.doctree.text(node.astext())
        raise nodes.SkipDeparture

    def depart_document(self, node: nodes.document):
        self.builder.source_map.seal(self.doctree)


def attrs_to_str(attrs: Dict) -> str:
    def default(v):
        return None

    return orjson.dumps(
        attrs,
        default=default,
        option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SORT_KEYS,
    ).decode("utf8")
