from typing import List

from docutils import nodes
from sphinx import addnodes

from sphinx_jsx.syntax.jsx.models import (
    JSON_NULL,
    JSFragment,
    JSONExpression,
    JSXElement,
    PseudoElement,
)

from .base import BaseJSXTranslator
from .pseudo import Pseudo


class Lists(PseudoElement):
    class ListItem(JSXElement):
        pass

    class BulletList(JSXElement):
        list_style: str = "disc"

    class EnumeratedList(JSXElement):
        list_style: str = "decimal"
        start: int = 1
        suffix: str = "."

    class DefinitionList(JSXElement):
        class Item(JSXElement):
            term: JSFragment = JSON_NULL
            classifiers: List[JSFragment] = []
            definition: JSFragment = JSON_NULL

        class _Term(PseudoElement):
            pass

        class _Classifier(PseudoElement):
            pass

        class _Definition(PseudoElement):
            pass

    class OptionList(JSXElement):
        class Item(JSXElement):
            options: List[JSFragment] = []
            description: JSFragment = JSON_NULL

        class Option(JSXElement):
            pass

        class _Group(PseudoElement):
            pass

        class _Option(PseudoElement):
            pass

    class FieldList(JSXElement):
        class Field(JSXElement):
            name: JSFragment = JSON_NULL
            body: JSFragment = JSON_NULL

        class _Name(PseudoElement):
            pass

        class _Body(PseudoElement):
            pass

    class HorizontalList(JSXElement):
        columns: List[JSFragment] = []

        class _Column(PseudoElement):
            pass


class ListMarkupTranslator(BaseJSXTranslator):
    def visit_bullet_list(self, node: nodes.bullet_list):
        elem = self.enter_nesting(node, Lists.BulletList())
        elem.list_style = node.get("bullet", "disc")

    def visit_enumerated_list(self, node: nodes.enumerated_list):
        self.enter_nesting(
            node,
            Lists.EnumeratedList(
                list_style=node.get("enumtype"),
                start=node.get("start") or 1,
                suffix=node.get("suffix"),
            ),
        )

    def visit_list_item(self, node: nodes.list_item):
        self.enter_nesting(node, Lists.ListItem())

    def visit_definition_list(self, node: nodes.definition_list):
        self.enter_nesting(node, Lists.DefinitionList())

    def visit_definition_list_item(self, node: nodes.definition_list_item):
        self.enter_nesting(node, Lists.DefinitionList.Item())

    def visit_term(self, node: nodes.term):
        self.enter_nesting(node, Lists.DefinitionList._Term())

    def visit_classifier(self, node: nodes.classifier):
        self.enter_nesting(node, Lists.DefinitionList._Classifier())

    def visit_definition(self, node: nodes.definition):
        self.enter_nesting(node, Lists.DefinitionList._Definition())

    def depart_definition_list_item(self, node: nodes.definition_list_item):
        elem = self.leave_nesting(node, Lists.DefinitionList.Item)
        elem.term = elem.remove_fragment(Lists.DefinitionList._Term)
        classifiers = elem.remove_children(Lists.DefinitionList._Classifier)
        elem.classifiers = [c.to_fragment() for c in classifiers]
        elem.definition = elem.remove_fragment(Lists.DefinitionList._Definition)

    def visit_field_list(self, node: nodes.field_list):
        self.enter_nesting(node, Lists.FieldList())

    def visit_field(self, node: nodes.field):
        self.enter_nesting(node, Lists.FieldList.Field())

    def visit_field_name(self, node: nodes.field_name):
        self.enter_nesting(node, Lists.FieldList._Name())

    def visit_field_body(self, node: nodes.field_body):
        self.enter_nesting(node, Lists.FieldList._Body())

    def depart_field(self, node: nodes.field):
        elem = self.leave_nesting(node, Lists.FieldList.Field)
        elem.name = elem.remove_fragment(Lists.FieldList._Name)
        elem.body = elem.remove_fragment(Lists.FieldList._Body)

    def visit_option_list(self, node: nodes.option_list):
        self.enter_nesting(node, Lists.OptionList())

    def visit_option_list_item(self, node: nodes.option_list_item):
        self.enter_nesting(node, Lists.OptionList.Item())

    def visit_option_group(self, node: nodes.option_group):
        self.enter_nesting(node, Lists.OptionList._Group())

    def visit_option(self, node: nodes.option):
        self.enter_nesting(node, Lists.OptionList._Option())

    def visit_option_string(self, node: nodes.option_string):
        self.append_child(node, JSONExpression(value=node.astext()))
        raise nodes.SkipNode

    def visit_option_argument(self, node: nodes.option_argument):
        self.append_child(node, JSONExpression(value=node.astext()))
        raise nodes.SkipNode

    def visit_description(self, node: nodes.description):
        self.enter_nesting(node, Pseudo._Description())

    def depart_option_list_item(self, node: nodes.option_list_item):
        elem = self.leave_nesting(node, Lists.OptionList.Item)
        for group in elem.remove_children(Lists.OptionList._Group):
            for opt in group.remove_children(Lists.OptionList._Option):
                option = Lists.OptionList.Option(children=opt.children)
                elem.options.append(option)
        elem.description = elem.remove_fragment(Pseudo._Description)

    def visit_hlist(self, node: addnodes.hlist):
        self.enter_nesting(node, Lists.HorizontalList())

    def visit_hlistcol(self, node: addnodes.hlistcol):
        self.enter_nesting(node, Lists.HorizontalList._Column())

    def depart_hlist(self, node: addnodes.hlist):
        elem = self.leave_nesting(node, Lists.HorizontalList)
        for column in elem.remove_children(Lists.HorizontalList._Column):
            elem.columns.append(column.to_fragment())
