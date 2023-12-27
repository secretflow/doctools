from dataclasses import dataclass
from typing import (
    Any,
    Dict,
    List,
    Optional,
    Set,
    Type,
    TypeVar,
    Union,
    final,
    overload,
)
from urllib.parse import urlsplit

from docutils.nodes import Element, Node, NodeVisitor, document
from loguru import logger

from sphinx_jsx.syntax.jsx.models import (
    EmptySelectionError,
    ESMExport,
    ESMImport,
    JSFragment,
    JSONExpression,
    JSXElement,
    React,
)
from sphinx_jsx.utils.logging import qualname

from .intrinsic import HTML, Intrinsic
from .references import Link

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

    @property
    def parent(self):
        return self.ancestors[-1].markup

    @final
    def add_id(self, origin: Element, markup: JSXElement) -> None:
        markup.ids.extend(origin.get("ids", []))

    @final
    def add_classnames(self, origin: Element, markup: JSXElement) -> None:
        markup.classnames.extend(origin.get("classes", []))

    @final
    def add_style(self, origin: Element, markup: JSXElement) -> None:
        style: str = origin.get("style")
        if not style:
            return
        class_name = f"styled-{len(self.stylesheet)}"
        self.stylesheet[class_name] = style
        markup.classnames.append(class_name)

    @final
    def add_reference(self, node: Union[str, Element]) -> Link:
        if isinstance(node, str):
            url = node

        elif "refid" in node:
            # target on the same page
            url = f'#{node["refid"]}'

        elif "reftarget" in node:
            # download link to local file
            url = node["reftarget"]

        elif "refuri" in node:
            # target on a different page or external
            url = node["refuri"] or "#"

        else:
            url = "#"

        try:
            parsed = urlsplit(url)
        except ValueError:
            parsed = urlsplit("#")

        if parsed.scheme:
            if parsed.scheme != "file":
                reftype = "external"
            else:
                reftype = "internal"
        elif parsed.path:
            reftype = "internal"
        else:
            reftype = "fragment"

        if reftype != "fragment":
            self.references.add(url)

        return Link(href=url, reftype=reftype)

    @final
    def add_export(self, name: str, value: Any):
        self.body.children.append(ESMExport(name=name, value=value))

    @final
    def append_child(self, origin: Optional[Element], markup: JSFragment):
        logger.bind(origin=origin).debug(self.log_ancestors(f"++ {repr(markup)}"))

        if origin and isinstance(markup, JSXElement):
            self.add_id(origin, markup)
            self.add_classnames(origin, markup)
            self.add_style(origin, markup)

        if Intrinsic.is_head_element(markup):
            self.head.children.append(markup)
        else:
            self.parent.children.append(markup)

        return markup

    @final
    def enter_nesting(self, origin: Element, markup: TElement) -> TElement:
        logger.bind(origin=origin).debug(self.log_ancestors(f"-> {repr(markup)}"))
        self.ancestors.append(ParentNode(origin, markup))
        return markup

    @overload
    def leave_nesting(self, origin: Element) -> Optional[JSXElement]:
        ...

    @overload
    def leave_nesting(self, origin: Element, of_type: Type[TElement]) -> TElement:
        ...

    @final
    def leave_nesting(
        self,
        origin: Element,
        of_type: Optional[Type[TElement]] = None,
    ):
        child = self.ancestors.pop()

        if child.origin is not origin:
            # this guarantees parity
            if of_type is not None:
                raise EmptySelectionError(self.parent, of_type)

            self.ancestors.append(child)
            return None

        logger.bind(origin=origin).debug(self.log_ancestors(f"<- {repr(child.markup)}"))

        self.append_child(origin, child.markup)

        if of_type is not None:
            if not isinstance(child.markup, of_type):
                raise EmptySelectionError(self.parent, of_type)

            return child.markup

        return child.markup

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

    def render(self, import_map: Optional[Dict[str, str]] = None):
        import_map = import_map or {}

        head = [*self.head.children]
        body = [*self.body.children]

        if self.stylesheet:
            stylesheet = "\n".join(
                f".{class_name} {{ {style} }}"
                for class_name, style in self.stylesheet.items()
            )
            head.append(HTML.style(children=[JSONExpression(value=stylesheet)]))

        for name, module in import_map.items():
            body.append(ESMImport(name=name, source=module))

        statements: List[JSFragment]

        if head:
            statements = [HTML.head(children=head), *body]
        else:
            statements = body

        return "\n\n".join((s.render() for s in statements))

    def log_ancestors(self, msg: str):
        try:
            ancestors = self.ancestors
        except AttributeError:
            path = "()"
        else:
            path = " > ".join(repr(t.markup) for t in ancestors)
        return f"{path} {msg}"
