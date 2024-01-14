from dataclasses import dataclass
from typing import (
    Dict,
    List,
    Set,
    TypeVar,
    final,
)

from docutils.nodes import Element, Node, NodeVisitor, document
from loguru import logger

from sphinx_jsx.syntax.jsx.models import (
    EmptySelectionError,
    JSXElement,
    React,
)
from sphinx_jsx.utils.logging import qualname

TElement = TypeVar("TElement", bound=JSXElement)
UElement = TypeVar("UElement", bound=JSXElement)


@dataclass
class ParentNode:
    origin: Element
    markup: JSXElement


class BaseJSXTranslator(NodeVisitor):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.head: React.Fragment
        self.fragments: Dict[str, React.Fragment]
        self.ancestors: List[ParentNode]

        self.references: Set[str] = set()
        self.stylesheet: Dict[str, str] = {}

    def visit_document(self, node: document):
        if hasattr(self, "body"):
            raise RuntimeError("Translator is not reusable")

        self.head = React.Fragment()
        self.body = React.Fragment()
        self.ancestors = [ParentNode(node, self.body)]

    def depart_document(self, node: document):
        self.ancestors.pop()

    @final
    def dispatch_visit(self, node: Node):
        # match Sphinx's default behavior
        for node_class in node.__class__.mro():
            method_name = f"visit_{node_class.__name__}"
            method = getattr(self, method_name, None)
            if not method:
                continue
            logger.bind(origin=node).debug(method_name)
            method(node)
            break
        else:
            logger.bind(origin=node).warning(
                "No visitor configured for {node_type}",
                node_type=qualname(type(node)),
            )

    @final
    def dispatch_departure(self, node: Node):
        # match Sphinx's default behavior, with special handling for EmptySelectionError
        node_type = qualname(type(node))
        for node_class in node.__class__.__mro__:
            method_name = f"depart_{node_class.__name__}"
            method = getattr(self, method_name, None)
            if not method:
                continue
            try:
                method(node)
                logger.bind(origin=node).debug(method_name)
            except EmptySelectionError:
                msg = f"{method_name} does not handle {node_type}"
                logger.bind(origin=node).debug(msg)
                continue
            else:
                break
        else:
            # default departure
            if isinstance(node, Element):
                logger.bind(origin=node).debug("unknown_departure")
                while self.leave_nesting(node) is not None:
                    pass

    @final
    def unknown_visit(self, node: Node):
        raise NotImplementedError(f"Unknown node: {node!r}")  # pragma: no cover

    @final
    def unknown_departure(self, node: Node):
        raise NotImplementedError(f"Unknown node: {node!r}")  # pragma: no cover
