from __future__ import annotations

from typing import Optional

from docutils import nodes

from sphinx_jsx.syntax.jsx.models import (
    JSON_NULL,
    JSFragment,
    JSXElement,
    PseudoElement,
)
from sphinx_jsx.utils.pydantic import update_forward_refs

from .base import BaseJSXTranslator
from .pseudo import Pseudo


class Table(JSXElement):
    caption: JSFragment = JSON_NULL

    class Colgroup(JSXElement):
        pass

    class Col(JSXElement):
        pass

    class Head(JSXElement):
        pass

    class Body(JSXElement):
        pass

    class Foot(JSXElement):
        pass

    class Row(JSXElement):
        pass

    class HeaderCell(JSXElement):
        colspan: Optional[str] = None
        rowspan: Optional[str] = None

    class DataCell(JSXElement):
        colspan: Optional[str] = None
        rowspan: Optional[str] = None

    class _Entry(PseudoElement):
        morecols: Optional[int] = None
        morerows: Optional[int] = None

    class _ColSpec(PseudoElement):
        width: Optional[str] = None


class TableMarkupTranslator(BaseJSXTranslator):
    def _get_span(self, more: Optional[int]):
        if not more:
            return None
        return str(more + 1)

    def visit_table(self, node: nodes.table):
        self.enter_nesting(node, Table())

    def visit_tgroup(self, node: nodes.tgroup):
        pass

    def visit_tabular_col_spec(self, node):
        raise nodes.SkipNode

    def visit_colspec(self, node: nodes.colspec):
        self.enter_nesting(node, Table._ColSpec(width=node.get("colwidth")))

    def visit_thead(self, node: nodes.thead):
        self.enter_nesting(node, Table.Head())

    def visit_tbody(self, node: nodes.tbody):
        self.enter_nesting(node, Table.Body())

    def visit_row(self, node: nodes.row):
        self.enter_nesting(node, Table.Row())

    def visit_entry(self, node: nodes.entry):
        self.enter_nesting(
            node,
            Table._Entry(
                morecols=node.get("morecols"),
                morerows=node.get("morerows"),
            ),
        )

    def depart_thead(self, node: nodes.thead):
        elem = self.leave_nesting(node, Table.Head)

        for row in elem.filter_children(Table.Row):

            def convert_to_th(e: Table._Entry) -> Table.HeaderCell:
                return Table.HeaderCell(
                    colspan=self._get_span(e.morecols),
                    rowspan=self._get_span(e.morerows),
                    children=e.children,
                )

            row.children = [*row.map_children(Table._Entry, convert_to_th)]

    def depart_tbody(self, node: nodes.tbody):
        elem = self.leave_nesting(node, Table.Body)

        for row in elem.filter_children(Table.Row):

            def convert_to_td(e: Table._Entry) -> Table.DataCell:
                return Table.DataCell(
                    colspan=self._get_span(e.morecols),
                    rowspan=self._get_span(e.morerows),
                    children=e.children,
                )

            row.children = [*row.map_children(Table._Entry, convert_to_td)]

    def depart_table(self, node: nodes.table):
        elem = self.leave_nesting(node, Table)
        elem.caption = elem.remove_fragment(Pseudo._Title)
        colspecs = elem.remove_children(Table._ColSpec)
        colgroup = Table.Colgroup()
        total_width = sum(int(col.width or 1) for col in colspecs)
        for col in colspecs:
            style = f"width: calc(100% * {col.width or 1} / {total_width});"
            elem = Table.Col()
            elem._style = style
            colgroup.children.append(elem)
        if colgroup.children:
            elem.children.insert(0, colgroup)


update_forward_refs(globals())
