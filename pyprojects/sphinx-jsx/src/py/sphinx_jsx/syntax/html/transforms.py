import uuid
from typing import Dict, List, TypeVar, cast

from bs4 import BeautifulSoup, NavigableString, Tag
from bs4.element import PreformattedString
from docutils.nodes import Element, Node, Text, paragraph, raw
from sphinx.transforms.post_transforms import SphinxPostTransform

from sphinx_jsx.syntax import rst

from .elements import html_element, html_fragment

RST_IN_HTML = "rst-in-html"

T = TypeVar("T", bound=Element)


def consume_soup(preserve: Dict[str, Element], tree: T, soup: Tag) -> T:
    for tag in soup:
        if isinstance(tag, PreformattedString):
            continue
        if isinstance(tag, NavigableString):
            tree.append(Text(str(tag)))
            continue
        if not isinstance(tag, Tag):
            continue
        if tag.name == "script":
            continue
        if tag.name in ("html", "head", "body"):
            consume_soup(preserve, tree, tag)
            continue
        if tag.name == RST_IN_HTML and tag["id"] in preserve:
            tree.append(preserve[str(tag["id"])])
            continue
        elem = html_element("", name=tag.name)
        elem.document = tree.document
        elem.source = tree.source
        elem.line = tree.line
        elem["ids"] = tag.get_attribute_list("id", [])
        elem["classes"] = tag.get_attribute_list("class", [])
        elem["style"] = tag.get("style")
        tag.attrs.pop("id", None)
        tag.attrs.pop("class", None)
        tag.attrs.pop("style", None)
        elem["attrs"] = {**tag.attrs}
        consume_soup(preserve, elem, tag)
        tree.append(elem)
    return tree


def inline_html_to_doctree(para: paragraph) -> paragraph:
    rst_nodes: Dict[str, Element] = {}
    raw_nodes: List[Node] = []

    for node in para.children:
        if isinstance(node, raw) and node["format"] == "html":
            raw_nodes.append(node)
        elif isinstance(node, Text):
            raw_nodes.append(node)
        elif isinstance(node, Element):
            key = str(uuid.uuid4())
            rst_nodes[key] = node
            raw_nodes.append(raw("", f'<{RST_IN_HTML} id="{key}">', format="html"))

    source = "".join(node.astext() for node in raw_nodes)
    soup = BeautifulSoup(source, "lxml")
    return consume_soup(rst_nodes, para.copy(), cast(Tag, soup.find("p")))


def html_block_to_doctree(raw: raw) -> html_fragment:
    soup = BeautifulSoup(raw.astext().strip(), "lxml")
    fragment = html_fragment(raw.rawsource)
    fragment.document = raw.document
    fragment.source = raw.source
    fragment.line = raw.line
    return consume_soup({}, fragment, soup)


class InlineHTMLTransform(SphinxPostTransform):
    default_priority = 999
    builders = ("jsx",)

    def run(self, **kwargs) -> None:
        for line in self.document.findall(paragraph):
            if not line.next_node(raw):
                continue
            rst.transforms.replace_self(line, inline_html_to_doctree(line))

        for block in self.document.findall(raw):
            if block["format"] == "html":
                rst.transforms.replace_self(block, html_block_to_doctree(block))
