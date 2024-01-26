from pathlib import Path
from typing import Dict, NamedTuple, Optional

import orjson
from docutils import nodes
from loguru import logger
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

        try:
            source_path = Path(document.attributes["source"])
        except KeyError as e:
            raise ValueError(f"document {document} does not have a source path") from e

        try:
            # FIXME: include?????
            source = source_path.read_text("utf8")
        except OSError as e:
            raise ValueError(f"could not read source file {source_path}") from e

        self.builder = builder
        self.ast = builder.bundler.make_document(source_path, source)

        # TODO: edge case ''
        self.lines = source.splitlines()
        self.range = Range(1, 1, 1, 1)

    def find_next_rawsource(
        self,
        rawsource: str,
    ) -> Optional[Range]:
        current_line = self.lines[self.range.start_line - 1]
        start_index = current_line.find(rawsource, self.range.start_column - 1)
        if start_index != -1:
            return Range(
                self.range.start_line,
                start_index + 1,
                self.range.start_line,
                start_index + 1 + len(rawsource),
            )
        next_line = self.range.start_line
        for lineno, line in enumerate(self.lines[next_line:], next_line):
            start_index = line.find(rawsource)
            if start_index != -1:
                return Range(
                    lineno + 1,
                    start_index + 1,
                    lineno + 1,
                    start_index + 1 + len(rawsource),
                )
        return None

    def visit_Element(self, node: nodes.Element):
        try:
            if node.line and node.line != self.range:
                self.range = Range(
                    node.line,
                    1,
                    node.line,
                    len(self.lines[node.line - 1]) + 1,
                )

            if not node.rawsource:
                position = self.range
            else:
                found_source = self.find_next_rawsource(str(node.rawsource))
                if found_source:
                    self.range = position = found_source
                else:
                    position = self.range
        except IndexError:
            position = self.range

        name = type(node).__name__
        props = dump_props(node.attributes)
        self.ast.element(name, props, position=position)
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
