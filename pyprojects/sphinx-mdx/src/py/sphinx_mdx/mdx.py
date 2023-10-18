from __future__ import annotations

import abc
import json
from contextvars import ContextVar
from itertools import chain, cycle
from textwrap import indent
from typing import Any, Dict, Iterable, List, Literal, Optional, Union, get_args

from pydantic import BaseModel
from ruamel.yaml import YAML as YAMLSerializer
from typing_extensions import TypeAlias, TypeGuard

yaml = YAMLSerializer(typ="safe")


def join_inline(*items: Any) -> str:
    return "".join((str(c) for c in items))


def join_items(*items: Any) -> str:
    return "\n".join((str(c) for c in items))


def join_blocks(*items: Any) -> str:
    return "\n\n".join((str(c) for c in items))


_markup_ul = ContextVar[cycle]("_markup_ul")
_markup_ol = ContextVar[cycle]("_markup_ol")
_markup_em = ContextVar[cycle]("_markup_em")
_markup_strong = ContextVar[cycle]("_markup_strong")


_MARKUP_CHARS = str.maketrans(
    {
        "*": r"\*",
        "_": r"\_",
        "`": r"\`",
        "~": r"\~",
        "[": r"\[",
        "]": r"\]",
        "(": r"\(",
        ")": r"\)",
        "{": r"\{",
        "}": r"\}",
        "<": r"\<",
        ">": r"\>",
        "\\": r"\\",
    }
)


AlignType = Literal["left", "right", "center", None]

ReferenceType = Literal["shortcut", "collapsed", "full"]

JSON: TypeAlias = Union[Dict, List, str, int, float, bool, None]

HeadingDepth = Literal[1, 2, 3, 4, 5, 6]


class Alternative(BaseModel, abc.ABC):
    alt: Optional[str] = None


class Association(BaseModel, abc.ABC):
    identifier: str
    label: Optional[str] = None


class Resource(BaseModel, abc.ABC):
    url: str
    title: Optional[str] = None


class Reference(Association, BaseModel, abc.ABC):
    referenceType: ReferenceType = "full"


class Literal_(BaseModel, abc.ABC):
    value: str

    @abc.abstractmethod
    def __str__(self) -> str:
        ...


class Parent(BaseModel, abc.ABC):
    children: List = []

    @abc.abstractmethod
    def __str__(self) -> str:
        ...


class Root(Parent):
    type: Literal["root"] = "root"
    children: List[TopLevelContent] = []

    def __str__(self) -> str:
        return join_blocks(*self.children)


class Paragraph(Parent):
    type: Literal["paragraph"] = "paragraph"
    children: List[PhrasingContent] = []

    def __str__(self) -> str:
        return join_inline(*self.children)


class Heading(Parent):
    type: Literal["heading"] = "heading"
    depth: HeadingDepth
    children: List[PhrasingContent] = []

    def __str__(self) -> str:
        markup = "#" * self.depth
        return f"{markup} {join_inline(*self.children)}"


class ThematicBreak(Literal_):
    type: Literal["thematicBreak"] = "thematicBreak"
    value: Any = ""

    def __str__(self) -> str:
        return "---"


class Blockquote(Parent):
    type: Literal["blockquote"] = "blockquote"
    children: List[Union[BlockContent, DefinitionContent]] = []

    def __str__(self) -> str:
        return indent(join_blocks(*self.children), "> ", lambda line: True)


class List_(Parent):
    type: Literal["list"] = "list"
    ordered: Optional[bool] = None
    start: Optional[int] = None
    spread: Optional[bool] = None
    children: List[ListItem] = []

    @staticmethod
    def _init_markup(reset: bool = False):
        try:
            _markup_ol.get()
            reset_ol = False
        except LookupError:
            reset_ol = True
        try:
            _markup_ul.get()
            reset_ul = False
        except LookupError:
            reset_ul = True
        if reset or reset_ol:
            _markup_ol.set(cycle((". ", ") ")))
        if reset or reset_ul:
            _markup_ul.set(cycle(("- ", "+ ", "* ")))

    def current_bullet(self) -> str:
        self._init_markup()
        if self.ordered:
            return next(_markup_ol.get())
        else:
            return next(_markup_ul.get())

    def __str__(self) -> str:
        markup = self.current_bullet()

        items: List[str] = []
        start = 1 if self.start is None else self.start

        for i, item in enumerate(self.children, start=start):
            list_content = str(item)

            if not self.ordered:
                list_content = indent(list_content, " " * 2, lambda line: True)
                list_content = f"{markup}{list_content[len(markup):]}"
            else:
                list_content = indent(list_content, " " * 3, lambda line: True)
                list_content = f"{i}{markup}{list_content[len(str(i)) + len(markup):]}"

            items.append(list_content)

        if self.spread:
            return join_blocks(*items)
        return join_items(*items)


class ListItem(Parent):
    type: Literal["listItem"] = "listItem"
    spread: Optional[bool] = None
    children: List[Union[BlockContent, DefinitionContent]] = []

    def __str__(self):
        content = join_blocks(*self.children)
        if self.spread:
            return f"{content}\n"
        return content


class Table(Parent):
    type: Literal["table"] = "table"
    align: Optional[List[AlignType]] = None
    children: List[TableContent] = []


class TableRow(Parent):
    type: Literal["tableRow"] = "tableRow"
    children: List[RowContent] = []


class TableCell(Parent):
    type: Literal["tableCell"] = "tableCell"
    children: List[PhrasingContent] = []


class HTML(Literal_):
    type: Literal["html"] = "html"

    def __str__(self) -> str:
        raise NotImplementedError


class CodeBlock(Literal_):
    type: Literal["code"] = "code"
    lang: Optional[str] = None
    meta: Optional[str] = None

    def __str__(self) -> str:
        fence = "```"
        while fence in self.value:
            fence += "`"
        return f"{fence}{self.lang or ''}\n{self.value}\n{fence}"


class YAML(BaseModel):
    type: Literal["yaml"] = "yaml"
    data: Dict

    def __str__(self) -> str:
        return f"---\n{yaml.dump(self.data)}\n---"


class Definition(Association, Resource, BaseModel):
    type: Literal["definition"] = "definition"

    def __str__(self) -> str:
        label = self.label or self.identifier
        label = label.translate(_MARKUP_CHARS)
        return f"[{label}]: {self.url}"


class FootnoteDefinition(Parent, Association):
    type: Literal["footnoteDefinition"] = "footnoteDefinition"
    children: List[BlockContent] = []


class Text(Literal_):
    type: Literal["text"] = "text"

    def __str__(self) -> str:
        if "\n" in self.value:
            return str(JSONLiteral(data=self.value))
        return self.value.translate(_MARKUP_CHARS)


class Emphasis(Parent):
    type: Literal["emphasis"] = "emphasis"
    children: List[PhrasingContent] = []

    @staticmethod
    def _init_markup(reset: bool = False):
        try:
            _markup_em.get()
            reset_em = False
        except LookupError:
            reset_em = True
        if reset or reset_em:
            _markup_em.set(cycle(("_", "*")))

    def __str__(self) -> str:
        self._init_markup()
        markup = next(_markup_em.get())
        content = join_inline(*self.children)
        return f"{markup}{content}{markup}"


class Strong(Parent):
    type: Literal["strong"] = "strong"
    children: List[PhrasingContent] = []

    @staticmethod
    def _init_markup(reset: bool = False):
        try:
            _markup_strong.get()
            reset_strong = False
        except LookupError:
            reset_strong = True
        if reset or reset_strong:
            _markup_strong.set(cycle(("**", "__")))

    def __str__(self) -> str:
        self._init_markup()
        markup = next(_markup_strong.get())
        content = join_inline(*self.children)
        return f"{markup}{content}{markup}"


class Delete(Parent):
    type: Literal["delete"] = "delete"
    children: List[PhrasingContent] = []

    def __str__(self) -> str:
        return f"~~{join_inline(*self.children)}~~"


class InlineCode(Literal_):
    type: Literal["inlineCode"] = "inlineCode"

    def __str__(self) -> str:
        markup = "`"
        while markup in self.value:
            markup += "`"
        return f"{markup}{self.value}{markup}"


class Break(BaseModel):
    type: Literal["break"] = "break"

    def __str__(self) -> str:
        return "\\\n"


class Link(Parent, Resource):
    type: Literal["link"] = "link"
    children: List[StaticPhrasingContent] = []

    def __str__(self) -> str:
        content = join_inline(*self.children)
        if self.title:
            return f"[{content}]({self.url} {json.dumps(self.title)})"
        return f"[{content}]({self.url})"


class Image(Resource, Alternative, BaseModel):
    type: Literal["image"] = "image"

    def __str__(self) -> str:
        alt = (self.alt or "").translate(_MARKUP_CHARS)
        if self.title:
            return f"![{alt}]({self.url} {json.dumps(self.title)})"
        return f"![{alt}]({self.url})"


class LinkReference(Parent, Reference):
    type: Literal["linkReference"] = "linkReference"
    children: List[StaticPhrasingContent] = []

    def __str__(self) -> str:
        content = join_inline(*self.children)
        return f"[{content}][{self.identifier}]"


class ImageReference(Reference, Alternative, BaseModel):
    type: Literal["imageReference"] = "imageReference"

    def __str__(self) -> str:
        alt = (self.alt or "").translate(_MARKUP_CHARS)
        return f"![{alt}][{self.identifier}]"


class FootnoteReference(Association, BaseModel):
    type: Literal["footnoteReference"] = "footnoteReference"

    def __str__(self) -> str:
        return f"[^{self.identifier}]"


class _MDXFlowExpression(Literal_):
    type: Literal["mdxFlowExpression"] = "mdxFlowExpression"

    def __str__(self) -> str:
        return f"{{{self.value}}}"


class _MDXTextExpression(Literal_):
    type: Literal["mdxTextExpression"] = "mdxTextExpression"

    def __str__(self) -> str:
        return f"{{{self.value}}}"


class _MDXJSXAttributeValueExpression(Literal_):
    type: Literal["mdxJsxAttributeValueExpression"] = "mdxJsxAttributeValueExpression"

    def __str__(self) -> str:
        return f"{{{self.value}}}"


class _MDXJSXExpressionAttribute(Literal_):
    type: Literal["mdxJsxExpressionAttribute"] = "mdxJsxExpressionAttribute"

    def __str__(self) -> str:
        return f"{{{self.value}}}"


class _MDXJSXAttribute(BaseModel):
    type: Literal["mdxJsxAttribute"] = "mdxJsxAttribute"
    name: str
    value: Optional[Union[_MDXJSXAttributeValueExpression, str]] = None

    def __str__(self) -> str:
        if self.value is None:
            return self.name
        return f"{self.name}={self.value}"


MDXJSXAttributeList = List[Union[_MDXJSXAttribute, _MDXJSXExpressionAttribute]]


class _MDXElement(Parent, abc.ABC):
    name: Union[str, None] = None
    attributes: MDXJSXAttributeList = []

    def format_start_tag(self) -> str:
        attrs = " ".join(str(a) for a in self.attributes)
        if self.name is None:
            if attrs:
                raise ValueError("attributes are unsupported on fragments")
            return "<>"
        if attrs:
            return f"<{self.name} {attrs}>"
        return f"<{self.name}>"

    def format_end_tag(self) -> str:
        if self.name is None:
            return "</>"
        return f"</{self.name}>"


class _MDXJSXFlowElement(_MDXElement):
    type: Literal["mdxJsxFlowElement"] = "mdxJsxFlowElement"
    children: List[Union[BlockContent, DefinitionContent]] = []

    def __str__(self) -> str:
        content = join_blocks(*self.children)
        if content:
            return join_items(
                self.format_start_tag(),
                indent(content, " " * 2, lambda line: True),
                self.format_end_tag(),
            )
        return join_items(self.format_start_tag(), self.format_end_tag())


class _MDXJSXTextElement(_MDXElement):
    type: Literal["mdxJsxTextElement"] = "mdxJsxTextElement"
    children: List[PhrasingContent] = []

    def __str__(self) -> str:
        content = join_inline(*self.children)
        return join_inline(
            self.format_start_tag(),
            content,
            self.format_end_tag(),
        )


class _MDXJSESM(Literal_):
    type: Literal["mdxjsEsm"] = "mdxjsEsm"

    def __str__(self) -> str:
        return self.value


class JSONLiteral(BaseModel):
    type: Literal["_mdxJSONLiteral"] = "_mdxJSONLiteral"
    data: Any

    def __str__(self) -> str:
        return f"{{{json.dumps(self.data)}}}"


class Tag(BaseModel, abc.ABC):
    name: str
    attributes: Dict[str, Any] = {}
    children: List[Content] = []

    def to_attr_list(self):
        attrs: MDXJSXAttributeList = []
        for k, v in self.attributes.items():
            if v is True:
                attrs.append(_MDXJSXAttribute(name=k))
            else:
                value = JSONLiteral(data=v)
                attrs.append(_MDXJSXAttribute(name=k, value=str(value)))
        return attrs


class InlineTag(Tag):
    type: Literal["_mdxInlineTag"] = "_mdxInlineTag"
    children: List[PhrasingContent] = []

    @classmethod
    def new(
        cls,
        name: str,
        classes: Iterable[str] = (),
        children=None,
        **kwargs: Any,
    ):
        if classes:
            kwargs["className"] = " ".join(classes)
        return cls(name=name, children=children or [], attributes=kwargs)

    def __str__(self) -> str:
        attrs = self.to_attr_list()
        elem = _MDXJSXTextElement(
            name=self.name,
            attributes=attrs,
            children=self.children,
        )
        return str(elem)


class BlockTag(Tag):
    type: Literal["_mdxBlockTag"] = "_mdxBlockTag"
    children: List[Union[BlockContent, DefinitionContent]] = []

    @classmethod
    def new(
        cls,
        name: str,
        classes: Iterable[str] = (),
        children=None,
        **kwargs: Any,
    ):
        if classes:
            kwargs["className"] = " ".join(classes)
        return cls(name=name, children=children or [], attributes=kwargs)

    def __str__(self) -> str:
        attrs = self.to_attr_list()
        elem = _MDXJSXFlowElement(
            name=self.name,
            attributes=attrs,
            children=self.children,
        )
        return str(elem)


class ESMExport(BaseModel):
    type: Literal["_mdxESMExport"] = "_mdxESMExport"
    name: str
    value: JSON

    def __str__(self) -> str:
        return f"export const {self.name} = {json.dumps(self.value)};"


StaticPhrasingContent = Union[
    Text,
    Emphasis,
    Strong,
    Delete,
    HTML,
    InlineCode,
    Break,
    Image,
    ImageReference,
    FootnoteReference,
    InlineTag,
    JSONLiteral,
    _MDXJSXTextElement,
    _MDXTextExpression,
]

PhrasingContent = Union[
    StaticPhrasingContent,
    Link,
    LinkReference,
]

BlockContent = Union[
    Paragraph,
    Heading,
    ThematicBreak,
    Blockquote,
    List_,
    Table,
    HTML,
    CodeBlock,
    JSONLiteral,
    BlockTag,
    _MDXJSXFlowElement,
    _MDXFlowExpression,
    _MDXJSESM,
]

DefinitionContent = Union[Definition, FootnoteDefinition]

ListContent: TypeAlias = ListItem

TableContent: TypeAlias = TableRow

RowContent: TypeAlias = TableCell

FrontmatterContent: TypeAlias = YAML

TopLevelContent = Union[
    BlockContent,
    FrontmatterContent,
    DefinitionContent,
    ESMExport,
]

Content = Union[
    TopLevelContent,
    ListContent,
    TableContent,
    RowContent,
    BlockContent,
    PhrasingContent,
]

JSXElement = Union[BlockTag, InlineTag]


for t in chain(get_args(Content), (Root,)):
    if issubclass(t, BaseModel):
        t.update_forward_refs()


def is_jsx_element(markup: Any) -> TypeGuard[JSXElement]:
    return isinstance(markup, (BlockTag, InlineTag))


def is_element_of_type(markup: Any, tag_name: str) -> TypeGuard[Content]:
    return is_jsx_element(markup) and markup.name == tag_name
