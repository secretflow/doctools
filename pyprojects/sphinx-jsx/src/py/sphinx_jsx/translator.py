from pathlib import Path
from typing import Dict

import orjson
from docutils import nodes
from loguru import logger
from sphinx.builders import Builder
from sphinx.util.docutils import SphinxTranslator


class SphinxJSXTranslator(SphinxTranslator):
    def __init__(self, document: nodes.document, builder: Builder) -> None:
        from .builder import SphinxJSXBuilder

        if not isinstance(builder, SphinxJSXBuilder):
            raise TypeError("builder must be a SphinxJSXBuilder")

        super().__init__(document, builder)

        try:
            source_path = Path(document.attributes["source"])
        except KeyError as e:
            raise ValueError(f"document {document} does not have a source path") from e

        try:
            source = source_path.read_text("utf8")
        except OSError as e:
            raise ValueError(f"could not read source file {source_path}") from e

        self.builder = builder
        self.ast = builder.bundler.make_document(source_path, source)

        self.current_line = 1
        self.current_column = 1

    def visit_Element(self, node: nodes.Element):
        name = type(node).__name__
        props = dump_props(node.attributes)
        self.ast.element(name, props, position=None)
        # TODO: source map by scanning for rawsource
        self.ast.enter()

    def depart_Element(self, node: nodes.Element):
        self.ast.exit()

    def visit_Text(self, node: nodes.Text):
        self.ast.text(node.astext())
        raise nodes.SkipDeparture


def dump_props(props: Dict) -> str:
    def default(v):
        logger.warning(f"cannot serialize {repr(v)} as props, it will be discarded")
        return None

    return orjson.dumps(
        props,
        default=default,
        option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SORT_KEYS,
    ).decode("utf8")
