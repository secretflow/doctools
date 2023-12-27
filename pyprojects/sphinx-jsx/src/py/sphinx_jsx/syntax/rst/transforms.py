from typing import List, Type, TypeVar

from docutils import nodes
from sphinx.transforms.post_transforms import SphinxPostTransform

from . import elements

T = TypeVar("T", bound=nodes.Element)


def replace_self(self: nodes.Element, other: nodes.Element) -> nodes.Element:
    self.replace_self(other)
    if other.source is None:
        other.source = self.source
    if other.line is None:
        other.line = self.line
    return other


def append_child(self: nodes.Element, child: nodes.Node) -> None:
    self.append(child)
    if child.source is None:
        child.source = self.source
    if child.line is None:
        child.line = self.line


def extend_children(self: nodes.Element, children: List[nodes.Node]) -> None:
    for child in children:
        append_child(self, child)


def wrap(container: T, inner: nodes.Element) -> T:
    replace_self(inner, container)
    append_child(container, inner)
    return container


def unwrap(self: nodes.Element):
    for child in self.children:
        if isinstance(child, nodes.Node):
            if child.source is None:
                child.source = self.source
            if child.line is None:
                child.line = self.line
    self.replace_self(self.children)


def pull(elem: T) -> T:
    elem.parent.remove(elem)
    return elem


def move_to(container: nodes.Element, inner: T) -> T:
    inner = pull(inner)
    append_child(container, inner)
    return inner


def specialize_to(to: Type[T], elem: nodes.Element) -> T:
    repl = to(elem.rawsource, **elem.attributes)
    replace_self(elem, repl)
    extend_children(repl, elem.children)
    return repl


class TypePromotionTransform(SphinxPostTransform):
    """
    Specialize generic containers to distinct node classes.

    This makes it easier to write tree visitors.
    """

    default_priority = 999
    builders = ("jsx",)

    def run(self, **kwargs) -> None:
        for node in self.document.findall(nodes.Element):
            classes = node.get("classes", [])

            if isinstance(node, nodes.literal):
                if "download" in classes:
                    # <literal> inside a download link
                    specialize_to(nodes.inline, node)
                    continue

                if "kbd" in classes:
                    # <literal> inside a keyboard key
                    specialize_to(elements.literal__kbd, node)
                    continue

            if isinstance(node, nodes.container):
                if node.get("literal_block"):
                    specialize_to(elements.container__code, node)
                    continue

            if isinstance(node, nodes.title):
                if isinstance(node.parent, nodes.section):
                    title = specialize_to(elements.title__heading, node)
                    title["ids"] = node.parent["ids"]
                    node.parent["ids"] = []
                    continue

            if isinstance(node, nodes.topic):
                if classes == ["contents"]:
                    specialize_to(elements.topic__outline, node)
                    continue

            if isinstance(node, nodes.inline):
                if "guilabel" in classes or "menuselection" in classes:
                    specialize_to(elements.literal__kbd, node)
                    continue
