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
    def __init__(self, document: nodes.document, builder: Builder) -> None:
        from .builder import SphinxJSXBuilder

        if not isinstance(builder, SphinxJSXBuilder):
            raise TypeError("builder must be a SphinxJSXBuilder")

        super().__init__(document, builder)

        self.builder = builder
        self.bundler = builder.bundler

    def visit_Element(self, node: nodes.Element):
        component = type(node).__name__
        attrs = attrs_to_str(node.attributes)
        file_name = str(node.source) if node.source else None
        line_number = node.line
        raw_source = str(node.rawsource) if node.rawsource else None

        self.bundler.chunk(
            component,
            attrs,
            file_name=file_name,
            line_number=line_number,
            raw_source=raw_source,
        )

    def depart_Element(self, node: nodes.Element): ...

    def visit_Text(self, node: nodes.Text):
        raise nodes.SkipDeparture


def attrs_to_str(attrs: Dict) -> str:
    def default(v):
        return None

    return orjson.dumps(
        attrs,
        default=default,
        option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SORT_KEYS,
    ).decode("utf8")
