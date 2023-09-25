"""
Implement https://github.com/syntax-tree/mdast.

Upstream: https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/mdast/index.d.ts
"""

from __future__ import annotations

from io import StringIO
from typing import Dict, List, Literal, Optional, TypedDict, Union

from ruamel import yaml
from typing_extensions import NotRequired

from . import unist
from .content_model import (
    BlockContent,
    Content,
    DefinitionContent,
    ListContent,
    PhrasingContent,
    RowContent,
    StaticPhrasingContent,
    TableContent,
)

# --- MIXINS ---

AlignType = Literal["left", "right", "center", None]

ReferenceType = Literal["shortcut", "collapsed", "full"]


class Alternative(TypedDict):
    alt: NotRequired[Optional[str]]


class Association(TypedDict):
    identifier: str
    label: NotRequired[Optional[str]]


class Resource(TypedDict):
    url: str
    title: NotRequired[Optional[str]]


class Reference(TypedDict):
    referenceType: ReferenceType  # noqa: N815


# --- ELEMENTS ---


class Parent(unist.Parent):
    children: "List[Content]"


class Literal_(unist.Literal_):
    value: str


class Root(Parent):
    type: Literal["root"]


class Paragraph(Parent):
    type: Literal["paragraph"]
    children: List[PhrasingContent]


class Heading(Parent):
    type: Literal["heading"]
    depth: Literal[1, 2, 3, 4, 5, 6]
    children: List[PhrasingContent]


class ThematicBreak(unist.Node):
    type: Literal["thematicBreak"]


class Blockquote(Parent):
    type: Literal["blockquote"]
    children: List[Union[BlockContent, DefinitionContent]]


class List_(Parent):
    type: Literal["list"]
    ordered: Optional[bool]
    start: Optional[int]
    spread: Optional[bool]
    children: List[ListContent]


class ListItem(Parent):
    type: Literal["listItem"]
    spread: NotRequired[Optional[bool]]
    children: List[Union[BlockContent, DefinitionContent]]


class Table(Parent):
    type: Literal["table"]
    align: NotRequired[Optional[List[AlignType]]]
    children: List[TableContent]


class TableRow(Parent):
    type: Literal["tableRow"]
    children: List[RowContent]


class TableCell(Parent):
    type: Literal["tableCell"]
    children: List[PhrasingContent]


class HTML(Literal_):
    type: Literal["html"]


class Code(Literal_):
    type: Literal["code"]
    lang: NotRequired[Optional[str]]
    meta: NotRequired[Optional[str]]


class YAML(Literal_):
    type: Literal["yaml"]


class Definition(unist.Node, Association, Resource):
    type: Literal["definition"]


class FootnoteDefinition(Parent, Association):
    type: Literal["footnoteDefinition"]
    children: List[BlockContent]


class Text(Literal_):
    type: Literal["text"]


class Emphasis(Parent):
    type: Literal["emphasis"]
    children: List[PhrasingContent]


class Strong(Parent):
    type: Literal["strong"]
    children: List[PhrasingContent]


class Delete(Parent):
    type: Literal["delete"]
    children: List[PhrasingContent]


class InlineCode(Literal_):
    type: Literal["inlineCode"]


class Break(unist.Node):
    type: Literal["break"]


class Link(Parent, Resource):
    type: Literal["link"]
    children: List[StaticPhrasingContent]


class Image(unist.Node, Resource, Alternative):
    type: Literal["image"]


class LinkReference(Parent, Reference):
    type: Literal["linkReference"]
    children: List[StaticPhrasingContent]


class ImageReference(unist.Node, Reference, Alternative):
    type: Literal["imageReference"]


# class Footnote(Parent):
# type: Literal["footnote"]


class FootnoteReference(unist.Node, Association):
    type: Literal["footnoteReference"]


def root() -> Root:
    return {"type": "root", "children": []}


def text(t: str) -> Text:
    return {"type": "text", "value": t}


def heading(depth: Literal[1, 2, 3, 4, 5, 6]) -> Heading:
    return {"type": "heading", "depth": depth, "children": []}


def paragraph() -> Paragraph:
    return {"type": "paragraph", "children": []}


def code_block(content: str, lang: str = "plaintext") -> Code:
    return {"type": "code", "value": content, "lang": lang}


def unordered_list() -> List_:
    return {
        "type": "list",
        "start": None,
        "spread": False,
        "ordered": False,
        "children": [],
    }


def ordered_list(start: Optional[int] = None) -> List_:
    return {
        "type": "list",
        "start": start,
        "spread": False,
        "ordered": True,
        "children": [],
    }


def list_item() -> ListItem:
    return {"type": "listItem", "spread": False, "children": []}


def thematic_break() -> ThematicBreak:
    return {"type": "thematicBreak"}


def blockquote() -> Blockquote:
    return {"type": "blockquote", "children": []}


def strong() -> Strong:
    return {"type": "strong", "children": []}


def emphasis() -> Emphasis:
    return {"type": "emphasis", "children": []}


def inline_code(content: str) -> InlineCode:
    return {"type": "inlineCode", "value": content}


def delete() -> Delete:
    return {"type": "delete", "children": []}


def link(url: str, title: Optional[str] = None) -> Link:
    return {"type": "link", "url": url, "title": title, "children": []}


def footnote(identifier: str) -> FootnoteDefinition:
    return {
        "type": "footnoteDefinition",
        "identifier": identifier,
        "label": identifier,
        "children": [],
    }


def footnote_ref(identifier: str) -> FootnoteReference:
    return {"type": "footnoteReference", "identifier": identifier, "label": identifier}


def image(url: str, *, title: Optional[str] = None, alt: Optional[str] = None) -> Image:
    return {"type": "image", "url": url, "title": title, "alt": alt}


def frontmatter(data: Dict) -> YAML:
    dumper = yaml.YAML(typ="safe")
    dumper.default_flow_style = False
    f = StringIO()
    dumper.dump(data, f)
    return {"type": "yaml", "value": f.getvalue().strip("\n")}


def raw_html(value: str) -> HTML:
    return {"type": "html", "value": value}


def table(align: Optional[List[AlignType]] = None) -> Table:
    return {"type": "table", "align": align, "children": []}


def table_row() -> TableRow:
    return {"type": "tableRow", "children": []}


def table_cell() -> TableCell:
    return {"type": "tableCell", "children": []}
