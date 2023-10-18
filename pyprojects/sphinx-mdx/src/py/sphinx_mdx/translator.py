from __future__ import annotations

from collections import ChainMap, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import (
    Any,
    Callable,
    DefaultDict,
    Dict,
    List,
    Optional,
    Protocol,
    Set,
    Type,
    TypeVar,
    cast,
)

from docutils import nodes
from more_itertools import first
from myst_parser.parsers import parse_html
from sphinx import addnodes
from sphinx.builders import Builder
from sphinx.util import logging
from sphinx.util.docutils import SphinxTranslator
from typing_extensions import TypeAlias

from . import mdx
from .scaffolding import Scaffolding

UNSAFE_INLINE_CHARS = "_-*`~[]\n"

logger = logging.getLogger(__name__)


class _ContainerNode(Protocol):
    children: List[Any]


@dataclass
class ParentNode:
    origin: nodes.Element
    markup: _ContainerNode


NodeType = TypeVar("NodeType", bound=nodes.Node)
NodeHandler: TypeAlias = Callable[[NodeType], Optional[bool]]

T = TypeVar("T")


class MDXTranslator(SphinxTranslator):
    def __init__(self, document: nodes.document, builder: Builder):
        super().__init__(document, builder)

        scaffolding = getattr(builder, "scaffolding", None)
        if not isinstance(scaffolding, Scaffolding):
            raise TypeError(f"unsupported builder {builder}")

        self.scaffolding = scaffolding

        self.document = document
        self.settings = document.settings

        self.root: mdx.Root
        self.ancestors: List[ParentNode]

        self.ids: Set[str] = set()
        self.section_depth: mdx.HeadingDepth = cast(mdx.HeadingDepth, 0)

        self.context_info: ChainMap[str, Any] = ChainMap()
        self.context_handlers: DefaultDict[str, List[NodeHandler]] = defaultdict(list)
        self.context_callbacks: Dict[nodes.Node, Callable] = {}

    @property
    def source_path(self) -> Path:
        return Path(self.document["source"])

    @property
    def docname(self) -> str:
        docname = self.builder.env.path2doc(self.document["source"])
        if docname is None:
            raise ValueError()
        return docname

    @property
    def output_path(self) -> Path:
        return self.scaffolding.get_output_path(self.docname)

    @property
    def parent(self):
        return self.ancestors[-1].markup

    @property
    def is_in_jsx(self):
        return any(mdx.is_jsx_element(p.markup) for p in self.ancestors)

    def contextualize(self, node: nodes.Node, *handlers: NodeHandler, **info: Any):
        for fn in handlers:
            self.context_handlers[fn.__name__].append(fn)

        self.context_info.maps.insert(0, info)

        def callback():
            for fn in handlers:
                self.context_handlers[fn.__name__].remove(fn)

            self.context_info.maps.remove(info)

        self.context_callbacks[node] = callback

    def dispatch_visit(self, node: nodes.Node):
        for node_class in node.__class__.__mro__:
            method_name = f"visit_{node_class.__name__}"
            handled = False
            for method in self.context_handlers[method_name]:
                if method(node) is not False:
                    handled = True
                    break
            if handled:
                break
        else:
            super().dispatch_visit(node)

    def dispatch_departure(self, node: nodes.Node):
        for node_class in node.__class__.__mro__:
            method_name = f"depart_{node_class.__name__}"
            handled = False
            for method in self.context_handlers[method_name]:
                if method(node) is not False:
                    handled = True
                    break
            if handled:
                break
        else:
            super().dispatch_departure(node)
        callback = self.context_callbacks.pop(node, None)
        if callback is not None:
            callback()

    def add_element_id(self, origin: nodes.Element, markup: mdx.JSXElement):
        primary_id: Optional[str] = first(origin["ids"], None)
        if not primary_id:
            return
        if primary_id in self.ids:
            # ensure ID uniqueness
            return
        id_ = markup.attributes.setdefault("id", primary_id)
        self.ids.add(id_)

    def append_child(
        self,
        origin: Optional[nodes.Element],
        markup: mdx.Content,
    ) -> bool:
        try:
            parent = self.parent
        except IndexError:
            return False
        if origin is not None and mdx.is_jsx_element(markup):
            self.add_element_id(origin, markup)
        parent.children.append(markup)
        return True

    def enter_nesting(self, origin: nodes.Element, markup: _ContainerNode):
        self.ancestors.append(ParentNode(origin, markup))
        return markup

    def leave_nesting(
        self,
        origin: nodes.Element,
        type_: Type[T] = mdx.Content,
        *,
        multiple: bool = False,
    ) -> T:
        child = self.ancestors.pop()
        if child.origin is not origin:
            # this guarantees parity
            self.ancestors.append(child)
            return cast(type_, child.markup)
        self.append_child(origin, cast(mdx.Content, child.markup))
        if multiple:
            return self.leave_nesting(origin, type_, multiple=True)
        return cast(type_, child.markup)

    def visit_Text(self, node: nodes.Text):
        self.append_child(None, mdx.Text(value=node.astext()))

    # === Frontmatter ===

    def add_export(self, name: str, value: Any):
        self.root.children.append(mdx.ESMExport(name=name, value=value))

    def visit_meta(self, node: nodes.Element):
        name = node.get("name")
        lang = node.get("lang")
        value = node.get("content")
        http_equiv = node.get("http-equiv")

        attributes: Dict = {"content": value}

        if name:
            attributes["name"] = name
        elif http_equiv:
            attributes["httpEquiv"] = http_equiv

        if lang:
            attributes["lang"] = lang

        meta_tag = mdx.BlockTag.new("meta", **attributes)
        self.append_child(node, meta_tag)

    def visit_document(self, node: nodes.document):
        self.root = mdx.Root()
        self.ancestors = [ParentNode(node, self.root)]
        if node.get("title"):
            self.add_export("title", node["title"])

    def depart_document(self, node: nodes.document):
        self.leave_nesting(node)
        assert not self.ancestors, (
            f"stack: {self.ancestors}\n"
            "nesting stack is not empty. this indicates"
            " unclosed elements, which is a bug.\n"
            f"file: {self.source_path}"
        )

    def visit_section(self, node: nodes.section):
        self.section_depth += 1  # pyright: ignore[reportGeneralTypeIssues]

    def depart_section(self, node: nodes.section):
        self.section_depth -= 1  # pyright: ignore[reportGeneralTypeIssues]

    # === Flow content ===

    def visit_title(self, node: nodes.title):
        if isinstance(node.parent, nodes.table):
            self.enter_nesting(node, mdx.BlockTag.new("caption"))
        elif isinstance(node.parent, nodes.sidebar):
            self.enter_nesting(node, mdx.BlockTag.new("p", classes=("sidebar-title",)))
        elif isinstance(node.parent, addnodes.compact_paragraph):
            self.enter_nesting(node, mdx.BlockTag.new("p", classes=("caption",)))
        elif isinstance(node.parent, (nodes.Admonition, nodes.topic)):
            self.context_info["admonition_title"] = node.astext()
            raise nodes.SkipNode
        else:
            if isinstance(node.parent, nodes.section):
                node["ids"].extend(node.parent["ids"])
            self.enter_nesting(node, mdx.Heading(depth=self.section_depth))

    def depart_title(self, node: nodes.title):
        self.leave_nesting(node)

    def visit_paragraph(self, node: nodes.paragraph):
        self.enter_nesting(node, mdx.Paragraph())

    def depart_paragraph(self, node: nodes.paragraph):
        # multiple: wrap up any potentially unclosed HTML tags
        # TODO: handle malformed HTML more gracefully
        self.leave_nesting(node, multiple=True)

    def visit_literal_block(self, node: nodes.literal_block):
        lang = node.get("language", "plaintext")
        if lang == "ipython3":
            lang = "python"
        content = node.astext().strip()
        self.append_child(node, mdx.CodeBlock(value=content, lang=lang))
        raise nodes.SkipNode

    def visit_doctest_block(self, node: nodes.doctest_block):
        node["language"] = "python"
        self.visit_literal_block(cast(nodes.literal_block, node))

    def visit_transition(self, node: nodes.transition):
        self.append_child(node, mdx.ThematicBreak())
        raise nodes.SkipNode

    # === Lists ===

    def visit_bullet_list(self, node: nodes.bullet_list):
        self.enter_nesting(node, mdx.List_(ordered=False))

    def depart_bullet_list(self, node: nodes.bullet_list):
        ul = self.leave_nesting(node, mdx.List_)
        if any(child.spread for child in ul.children):
            ul.spread = True

    def visit_enumerated_list(self, node: nodes.enumerated_list):
        self.enter_nesting(node, mdx.List_(ordered=True, start=node.get("start")))

    def depart_enumerated_list(self, node: nodes.enumerated_list):
        ol = self.leave_nesting(node, mdx.List_)
        if any(child.spread for child in ol.children):
            ol.spread = True

    def visit_list_item(self, node: nodes.list_item):
        self.enter_nesting(node, mdx.ListItem())

    def depart_list_item(self, node: nodes.list_item):
        list_item = self.leave_nesting(node, mdx.ListItem)
        if any(
            isinstance(child, mdx.List_) and child.ordered
            for child in list_item.children
        ):
            list_item.spread = True

    def visit_definition_list(self, node: nodes.definition_list):
        self.enter_nesting(node, mdx.BlockTag.new("DefinitionList"))
        self.enter_nesting(node, mdx.BlockTag.new("dl"))

    def depart_definition_list(self, node: nodes.definition_list):
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_definition_list_item(self, node: nodes.definition_list_item):
        self.enter_nesting(node, mdx.BlockTag.new("dt"))

    def depart_definition_list_item(self, node: nodes.definition_list_item):
        pass

    def visit_term(self, node: nodes.term):
        self.enter_nesting(node, mdx.InlineTag.new("DefinitionList.Term"))
        self.enter_nesting(node, mdx.InlineTag.new("span"))

    def depart_term(self, node: nodes.term):
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_classifier(self, node: nodes.classifier):
        self.enter_nesting(node, mdx.InlineTag.new("em", classes=("classifier",)))
        self.append_child(node, mdx.Text(value=" : "))

    def depart_classifier(self, node: nodes.classifier):
        self.leave_nesting(node)

    def visit_definition(self, node: nodes.definition):
        self.leave_nesting(node.parent)  # close <dt />
        self.enter_nesting(node, mdx.BlockTag.new("dd"))

    def depart_definition(self, node: nodes.definition):
        self.leave_nesting(node)

    def visit_field_list(self, node: nodes.field_list):
        self.enter_nesting(node, mdx.BlockTag.new("FieldList"))
        self.enter_nesting(node, mdx.BlockTag.new("dl"))
        self.contextualize(node)

    def depart_field_list(self, node: nodes.field_list):
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_field(self, node: nodes.field):
        # treat as transparent
        pass

    def depart_field(self, node: nodes.field):
        pass

    def visit_field_name(self, node: nodes.field_name):
        self.enter_nesting(node, mdx.BlockTag.new("dt", classes=("field-name",)))
        self.context_info["field_section"] = node.astext()

    def depart_field_name(self, node: nodes.field_name):
        self.leave_nesting(node)

    def visit_field_body(self, node: nodes.field_body):
        self.enter_nesting(node, mdx.BlockTag.new("dd", classes=("field-body",)))

    def depart_field_body(self, node: nodes.field_body):
        self.leave_nesting(node)

    def visit_option_list(self, node: nodes.option_list):
        self.enter_nesting(node, mdx.BlockTag.new("OptionList"))
        self.enter_nesting(node, mdx.BlockTag.new("dl"))

    def depart_option_list(self, node: nodes.option_list):
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_option_list_item(self, node: nodes.option_list_item):
        pass

    def depart_option_list_item(self, node: nodes.option_list_item):
        pass

    def visit_option_group(self, node: nodes.option_group):
        self.enter_nesting(node, mdx.BlockTag.new("dt"))

    def depart_option_group(self, node: nodes.option_group):
        self.leave_nesting(node)

    def visit_option(self, node: nodes.option):
        if node.parent.index(node) > 0:
            self.append_child(node, mdx.Text(value=", "))
        self.enter_nesting(node, mdx.InlineTag.new("span", classes=("option",)))

    def depart_option(self, node: nodes.option):
        self.leave_nesting(node)

    def visit_option_string(self, node: nodes.option_string):
        self.enter_nesting(node, mdx.InlineTag.new("span"))
        self.append_child(node, mdx.Text(value=node.astext()))
        self.leave_nesting(node)
        raise nodes.SkipNode

    def visit_option_argument(self, node: nodes.option_argument):
        self.enter_nesting(node, mdx.InlineTag.new("span", classes=("argument",)))
        self.append_child(node, mdx.Text(value=node.astext()))
        self.leave_nesting(node)
        raise nodes.SkipNode

    def visit_description(self, node: nodes.description):
        self.enter_nesting(node, mdx.BlockTag.new("dd"))

    def depart_description(self, node: nodes.description):
        self.leave_nesting(node)

    # === Blockquotes ===

    def visit_block_quote(self, node: nodes.block_quote):
        self.enter_nesting(node, mdx.Blockquote())

    def depart_block_quote(self, node: nodes.block_quote):
        self.leave_nesting(node)

    def visit_line_block(self, node: nodes.line_block):
        self.enter_nesting(node, mdx.BlockTag.new("LineBlock"))

    def depart_line_block(self, node: nodes.line_block):
        self.leave_nesting(node)

    def visit_line(self, node: nodes.line):
        if not node.children:
            self.append_child(node, mdx.InlineTag.new("br"))
            raise nodes.SkipNode

        self.enter_nesting(node, mdx.Paragraph())

    def depart_line(self, node: nodes.line):
        self.leave_nesting(node)

    def visit_attribution(self, node: nodes.attribution):
        self.enter_nesting(node, mdx.InlineTag.new("em", classes=("attribution",)))
        self.append_child(node, mdx.Text(value="-- "))

    def depart_attribution(self, node: nodes.attribution):
        self.leave_nesting(node)

    # === Figures ===

    def visit_figure(self, node: nodes.figure):
        self.enter_nesting(node, mdx.BlockTag.new("figure"))

    def depart_figure(self, node: nodes.figure):
        if mdx.is_element_of_type(self.parent, "figcaption"):
            caption = node.next_node(nodes.caption)
            self.leave_nesting(caption)
        self.leave_nesting(node)

    def visit_caption(self, node: nodes.caption):
        self.enter_nesting(node, mdx.BlockTag.new("figcaption"))

    def depart_caption(self, node: nodes.caption):
        self.leave_nesting(node)

    def visit_legend(self, node: nodes.legend):
        pass

    def depart_legend(self, node: nodes.legend):
        pass

    # === Inline content ===

    def visit_inline(self, node: nodes.inline):
        if self.is_in_jsx:
            self.enter_nesting(node, mdx.InlineTag.new("span"))

    def depart_inline(self, node: nodes.inline):
        self.leave_nesting(node)

    def visit_strong(self, node: nodes.strong):
        self.enter_nesting(node, mdx.InlineTag.new("strong"))

    def depart_strong(self, node: nodes.strong):
        self.leave_nesting(node)

    def visit_emphasis(self, node: nodes.emphasis):
        self.enter_nesting(node, mdx.InlineTag.new("em"))

    def depart_emphasis(self, node: nodes.emphasis):
        self.leave_nesting(node)

    def visit_literal(self, node: nodes.literal):
        classes = node.get("classes", [])
        if node.get("language"):
            # inline code with language support
            elem = mdx.InlineTag.new("code", **{"data-language": node.get("language")})
            self.enter_nesting(node, elem)
            self.append_child(node, mdx.Text(value=node.astext()))
            self.leave_nesting(node)
        elif "kbd" in classes:
            # keyboard input
            self.enter_nesting(node, mdx.InlineTag.new("kbd"))
            self.append_child(node, mdx.Text(value=node.astext()))
            self.leave_nesting(node)
        elif "download" in classes:
            # download link (already wrapped in <a />)
            self.append_child(node, mdx.Text(value=node.astext()))
        else:
            self.append_child(node, mdx.InlineCode(value=node.astext()))
        raise nodes.SkipNode

    def visit_subscript(self, node: nodes.subscript):
        self.enter_nesting(node, mdx.InlineTag.new("sub"))

    def depart_subscript(self, node: nodes.subscript):
        self.leave_nesting(node)

    def visit_superscript(self, node: nodes.superscript):
        self.enter_nesting(node, mdx.InlineTag.new("sup"))

    def depart_superscript(self, node: nodes.superscript):
        self.leave_nesting(node)

    # === References ===

    def visit_reference(self, node: nodes.reference):
        # From HTML5Translator
        url: str

        if "refid" in node:
            url = f'#{node["refid"]}'

        elif "refuri" in node:
            refuri = node["refuri"]

            if not refuri:
                # empty, probably invalid
                url = "#"

            if self.scaffolding.is_external_url(refuri):
                url = refuri

            else:
                url = self.scaffolding.get_output_path_from_refuri(refuri)

        else:
            return

        self.enter_nesting(node, mdx.Link(url=url, title=node.get("reftitle")))

    def depart_reference(self, node: nodes.reference):
        self.leave_nesting(node)

    def visit_footnote_reference(self, node: nodes.footnote_reference):
        self.enter_nesting(node, mdx.InlineTag.new("sup"))
        self.visit_reference(cast(nodes.reference, node))

    def depart_footnote_reference(self, node: nodes.footnote_reference):
        self.depart_reference(cast(nodes.reference, node))
        self.leave_nesting(node)

    def visit_footnote(self, node: nodes.footnote):
        label_node = node.next_node(nodes.label)
        if label_node:
            label = label_node.astext()
        else:
            label = None
        self.enter_nesting(
            node,
            mdx.BlockTag.new("Footnote", label=label, backrefs=node.get("backrefs")),
        )

    def depart_footnote(self, node: nodes.footnote):
        self.leave_nesting(node)

    def visit_target(self, node: nodes.Element):
        if not len(node):
            return
        self.enter_nesting(node, mdx.InlineTag.new("span"))

    def depart_target(self, node: nodes.Element):
        self.leave_nesting(node)

    def visit_citation(self, node: nodes.citation):
        self.visit_footnote(cast(nodes.footnote, node))

    def depart_citation(self, node: nodes.citation):
        self.depart_footnote(cast(nodes.footnote, node))

    def visit_label(self, node: nodes.label):
        raise nodes.SkipNode

    def visit_image(self, node: nodes.image):
        src = self.scaffolding.add_image(self.docname, node)
        title = node.get("title", None)
        alt = node.get("alt", None)
        image = mdx.Image(url=src, title=title, alt=alt)
        self.append_child(node, image)
        raise nodes.SkipNode

    def visit_download_reference(self, node: addnodes.download_reference):
        uri = self.scaffolding.add_downloadable_file(self.docname, node)
        self.enter_nesting(node, mdx.InlineTag.new("a", href=uri, download=True))

    def depart_download_reference(self, node: addnodes.download_reference):
        self.leave_nesting(node)

    # === Sphinx: standard admonitions ===

    def visit_Admonition(self, node: nodes.Element):  # noqa: N802
        name = type(node).__name__
        title = node.get("title")
        self.enter_nesting(node, mdx.BlockTag.new("Container", type=name, title=title))
        self.contextualize(node)

    def depart_Admonition(self, node: nodes.Element):  # noqa: N802
        container = self.leave_nesting(node, mdx.BlockTag)
        if title := self.context_info.get("admonition_title"):
            container.attributes.setdefault("title", title)

    def visit_versionmodified(self, node: addnodes.versionmodified):
        if node["type"] == "versionadded":
            name = "info"
            title = "Version added"
        elif node["type"] == "versionchanged":
            name = "info"
            title = "Version changed"
        elif node["type"] == "deprecated":
            name = "warning"
            title = "Deprecated"
        else:
            return
        self.enter_nesting(node, mdx.BlockTag.new("Container", type=name, title=title))

    def depart_versionmodified(self, node: addnodes.versionmodified):
        self.leave_nesting(node)

    def visit_seealso(self, node: addnodes.seealso):
        self.enter_nesting(
            node, mdx.BlockTag.new("Container", type="info", title="See also:")
        )

    def depart_seealso(self, node: addnodes.seealso):
        self.leave_nesting(node)

    # === Sphinx: Horizontal lists ===

    def visit_hlist(self, node: addnodes.hlist) -> None:
        self.enter_nesting(node, mdx.BlockTag.new("HorizontalList"))
        self.enter_nesting(node, mdx.BlockTag.new("table"))
        self.enter_nesting(node, mdx.BlockTag.new("tr"))

    def depart_hlist(self, node: addnodes.hlist) -> None:
        self.leave_nesting(node)
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_hlistcol(self, node: addnodes.hlistcol) -> None:
        self.enter_nesting(node, mdx.BlockTag.new("td"))

    def depart_hlistcol(self, node: addnodes.hlistcol) -> None:
        self.leave_nesting(node)

    # === Sphinx: symbol/signature ===

    def visit_desc(self, node: addnodes.desc):
        domain: Optional[str] = node.attributes.get("domain")
        obj_type: Optional[str] = node.attributes.get("desctype")

        elem = mdx.BlockTag.new("Outline", domain=domain, objectType=obj_type)
        self.enter_nesting(node, elem)
        self.contextualize(node)

    def depart_desc(self, node: addnodes.desc):
        element = self.leave_nesting(node, mdx.BlockTag)
        info = {**self.context_info}
        element.attributes.update(info)

    def visit_desc_signature(self, node: addnodes.desc_signature):
        elem = mdx.BlockTag.new("Outline.Signature", fullname=node.get("fullname"))
        self.enter_nesting(node, elem)

        self.context_info["target"] = first(node.get("ids"), None)
        self.context_info["module"] = node.get("module")
        self.context_info["fullname"] = node.get("fullname")

    def depart_desc_signature(self, node: addnodes.desc_signature):
        self.leave_nesting(node)

    def visit_desc_name(self, node: addnodes.desc_name):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.Name"))

    def depart_desc_name(self, node: addnodes.desc_name):
        self.leave_nesting(node)

    def visit_desc_addname(self, node: addnodes.desc_addname):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.Prefix"))

    def depart_desc_addname(self, node: addnodes.desc_addname):
        self.leave_nesting(node)

    def visit_desc_annotation(self, node: addnodes.desc_annotation):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.Keyword"))

    def depart_desc_annotation(self, node: addnodes.desc_annotation):
        self.leave_nesting(node)

    def visit_desc_signature_line(self, node: addnodes.desc_signature_line):
        self.enter_nesting(node, mdx.Paragraph())

    def depart_desc_signature_line(self, node: addnodes.desc_signature_line):
        self.leave_nesting(node)

    first_param: int
    optional_param_level: int
    required_params_left: int
    param_separator: str

    def visit_desc_parameterlist(self, node: addnodes.desc_parameterlist):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.ParameterList"))
        self.enter_nesting(node, mdx.InlineTag.new("span"))
        self.append_child(node, mdx.Text(value="("))
        self.leave_nesting(node)

        # from sphinx.writers.html5.HTML5Translator
        self.first_param = 1
        self.optional_param_level = 0
        self.required_params_left = sum(
            [isinstance(c, addnodes.desc_parameter) for c in node.children]
        )
        self.param_separator = node.child_text_separator

        self.context_info["parameters"] = []

    def depart_desc_parameterlist(self, node: addnodes.desc_parameterlist):
        self.enter_nesting(node, mdx.InlineTag.new("span"))
        self.append_child(node, mdx.Text(value=")"))
        self.leave_nesting(node)
        self.leave_nesting(node)

    def visit_desc_parameter(self, node: addnodes.desc_parameter):
        if self.first_param:
            self.first_param = 0
        elif not self.required_params_left:
            self.append_child(node, mdx.Text(value=self.param_separator))
        if self.optional_param_level == 0:
            self.required_params_left -= 1

        self.enter_nesting(node, mdx.InlineTag.new("Outline.Parameter"))

        name = node.next_node(addnodes.desc_sig_name)
        if name is not None:
            self.context_info["parameters"].append(name.astext())

    def depart_desc_parameter(self, node: addnodes.desc_parameter):
        self.leave_nesting(node)
        if self.required_params_left:
            self.append_child(node, mdx.Text(value=self.param_separator))

    def visit_desc_optional(self, node: addnodes.desc_optional) -> None:
        self.optional_param_level += 1
        self.append_child(node, mdx.Text(value="["))

    def depart_desc_optional(self, node: addnodes.desc_optional) -> None:
        self.optional_param_level -= 1
        self.append_child(node, mdx.Text(value="]"))

    def visit_desc_sig_name(self, node: addnodes.desc_sig_name):
        self.enter_nesting(node, mdx.InlineTag.new("span", classes=("name",)))

    def depart_desc_sig_name(self, node: addnodes.desc_sig_name):
        self.leave_nesting(node)

    def visit_desc_sig_space(self, node: addnodes.desc_sig_space):
        self.enter_nesting(node, mdx.InlineTag.new("span"))
        self.append_child(node, mdx.JSONLiteral(data=" "))
        self.leave_nesting(node)
        raise nodes.SkipNode

    def visit_desc_sig_punctuation(self, node: addnodes.desc_sig_punctuation):
        self.enter_nesting(node, mdx.InlineTag.new("span"))

    def depart_desc_sig_punctuation(self, node: addnodes.desc_sig_punctuation):
        self.leave_nesting(node)

    def visit_desc_sig_operator(self, node: addnodes.desc_sig_operator):
        self.enter_nesting(node, mdx.InlineTag.new("span"))

    def depart_desc_sig_operator(self, node: addnodes.desc_sig_operator):
        self.leave_nesting(node)

    def visit_desc_inline(self, node: addnodes.desc_inline):
        self.enter_nesting(node, mdx.InlineTag.new("code"))

    def depart_desc_inline(self, node: addnodes.desc_inline):
        self.leave_nesting(node)

    def visit_desc_type(self, node: addnodes.desc_type):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.TypeAnnotation"))

    def depart_desc_type(self, node: addnodes.desc_type):
        self.leave_nesting(node)

    def visit_desc_returns(self, node: addnodes.desc_returns):
        self.enter_nesting(node, mdx.InlineTag.new("Outline.ReturnType"))
        self.enter_nesting(node, mdx.InlineTag.new("span"))
        self.append_child(node, mdx.JSONLiteral(data=" → "))
        self.leave_nesting(node)

    def depart_desc_returns(self, node: addnodes.desc_returns):
        self.leave_nesting(node)

    def visit_desc_content(self, node: addnodes.desc_content):
        self.enter_nesting(node, mdx.BlockTag.new("Outline.Content"))

        description = node.next_node(nodes.paragraph)
        if description:
            self.context_info["description"] = description.astext()

        def visit_literal_strong(node: addnodes.literal_strong):
            if self.context_info.get("field_section") != "Parameters":
                return False
            self.enter_nesting(node, mdx.InlineTag.new("Outline.ParameterTarget"))

        self.contextualize(node, visit_literal_strong)

    def depart_desc_content(self, node: addnodes.desc_content):
        self.leave_nesting(node)

    # === Tables ===

    def visit_table(self, node: nodes.table):
        self.enter_nesting(node, mdx.BlockTag.new("table"))

    def depart_table(self, node: nodes.table):
        self.leave_nesting(node)

    def visit_tgroup(self, node: nodes.tgroup):
        pass

    def depart_tgroup(self, node: nodes.tgroup):
        pass

    def visit_colspec(self, node: nodes.colspec):
        # colspecs where originally used to carry the width of a column
        # but this is now deprecated in favor of CSS
        pass

    def depart_colspec(self, node: nodes.colspec):
        pass

    def visit_tabular_col_spec(self, node: addnodes.tabular_col_spec):
        pass

    def depart_tabular_col_spec(self, node: addnodes.tabular_col_spec):
        pass

    def visit_thead(self, node: nodes.thead):
        self.enter_nesting(node, mdx.BlockTag.new("thead"))

    def depart_thead(self, node: nodes.thead):
        self.leave_nesting(node)

    def visit_tbody(self, node: nodes.tbody):
        self.enter_nesting(node, mdx.BlockTag.new("tbody"))

    def depart_tbody(self, node: nodes.tbody):
        self.leave_nesting(node)

    def visit_row(self, node: nodes.row):
        attrs: Dict[str, Any] = {}
        if node.get("morecols"):
            attrs["colSpan"] = int(node["morecols"]) + 1
        if node.get("morerows"):
            attrs["rowSpan"] = int(node["morerows"]) + 1
        self.enter_nesting(node, mdx.BlockTag.new("tr", **attrs))

    def depart_row(self, node: nodes.row):
        self.leave_nesting(node)

    def visit_entry(self, node: nodes.entry):
        attrs: Dict[str, Any] = {}
        if node.get("morecols"):
            attrs["colSpan"] = int(node["morecols"]) + 1
        if node.get("morerows"):
            attrs["rowSpan"] = int(node["morerows"]) + 1
        self.enter_nesting(node, mdx.BlockTag.new("td", **attrs))

    def depart_entry(self, node: nodes.entry):
        self.leave_nesting(node)

    # === Sidebar ===

    def visit_sidebar(self, node: nodes.sidebar):
        self.enter_nesting(node, mdx.BlockTag.new("aside"))

    def depart_sidebar(self, node: nodes.sidebar):
        self.leave_nesting(node)

    def visit_subtitle(self, node: nodes.subtitle):
        self.enter_nesting(node, mdx.BlockTag.new("p", classes=("sidebar-subtitle",)))

    def depart_subtitle(self, node: nodes.subtitle):
        self.leave_nesting(node)

    def visit_rubric(self, node: nodes.rubric):
        self.enter_nesting(node, mdx.BlockTag.new("p", classes=("sidebar-rubric",)))

    def depart_rubric(self, node: nodes.rubric):
        self.leave_nesting(node)

    # === Semantic text markup ===

    def visit_title_reference(self, node: nodes.title_reference):
        self.enter_nesting(node, mdx.InlineTag.new("cite"))

    def depart_title_reference(self, node: nodes.title_reference):
        self.leave_nesting(node)

    def visit_abbreviation(self, node: nodes.abbreviation):
        self.enter_nesting(node, mdx.InlineTag.new("abbr"))

    def depart_abbreviation(self, node: nodes.abbreviation):
        self.leave_nesting(node)

    def visit_acronym(self, node: nodes.acronym):
        self.visit_abbreviation(cast(nodes.abbreviation, node))

    def depart_acronym(self, node: nodes.acronym):
        self.depart_abbreviation(cast(nodes.abbreviation, node))

    # === Semantic containers ===

    def visit_compound(self, node: nodes.compound):
        if "toctree-wrapper" in node.get("classes", ()):
            self.enter_nesting(node, mdx.BlockTag.new("TableOfContents"))

    def depart_compound(self, node: nodes.compound):
        self.leave_nesting(node)

    def visit_glossary(self, node: addnodes.glossary):
        pass

    def depart_glossary(self, node: addnodes.glossary):
        pass

    # === Math ===

    def visit_math(self, node: nodes.math):
        self.enter_nesting(node, mdx.InlineTag.new("InlineMath", math=node.astext()))
        self.leave_nesting(node)
        raise nodes.SkipNode

    def visit_math_block(self, node: nodes.math_block):
        self.enter_nesting(node, mdx.BlockTag.new("Math", math=node.astext()))
        self.leave_nesting(node)
        raise nodes.SkipNode

    # === Raw ===

    def visit_raw(self, node: nodes.raw):
        # TODO: use servo
        if node.get("format") != "html":
            raise nodes.SkipNode

        def maybe_html(node: nodes.Element):
            try:
                return self.mdclient.html_to_tree(node.astext())
            except ValueError as e:
                logger.error("Malformed HTML: %s", node.astext(), location=node)
                raise nodes.SkipNode from e

        if isinstance(node.parent, nodes.paragraph):
            # We are inline
            # opening and closing tags are separated into
            # two different raw nodes
            try:
                tag = next(parse_html.tokenize_html(node.astext()).walk())

            except StopIteration:
                # empty AST, assume to be a closing tag
                self.leave_nesting(node.parent)

            else:
                # convert the opening tag to a (self-closing) JSX element
                # FIXME: no guarantee that this won't raise in the future
                transformed = maybe_html(node)

                # XHTML tags are considered "startendtags" in the parser
                if isinstance(tag, (parse_html.VoidTag, parse_html.XTag)):
                    for elem in transformed["children"]:
                        self.append_child(node.parent, elem)
                        break

                elif isinstance(tag, parse_html.Tag):
                    for elem in transformed["children"]:
                        self.enter_nesting(node.parent, elem)
                        break

        else:
            # We are a block element, treat the entire node as HTML
            transformed = maybe_html(node)
            for elem in transformed["children"]:
                self.append_child(node, elem)

        raise nodes.SkipNode

    def visit_container(self, node: nodes.container):
        if node.next_node(nodes.caption, descend=True, siblings=False):
            self.enter_nesting(node, mdx.BlockTag.new("figure"))
        else:
            self.enter_nesting(node, mdx.BlockTag.new("div"))

    def depart_container(self, node: nodes.container):
        self.leave_nesting(node)

    def visit_comment(self, node: nodes.comment):
        raise nodes.SkipNode

    def visit_topic(self, node: nodes.topic):
        self.enter_nesting(node, mdx.BlockTag.new("Container", type="info"))

    def depart_topic(self, node: nodes.topic):
        container = self.leave_nesting(node, mdx.BlockTag)
        if title := self.context_info.get("admonition_title"):
            container.attributes.setdefault("title", title)

    def visit_problematic(self, node: nodes.problematic):
        self.enter_nesting(node, mdx.InlineTag.new("del"))

    def depart_problematic(self, node: nodes.problematic):
        self.leave_nesting(node)

    def visit_substitution_definition(self, node: nodes.substitution_definition):
        # treat as transparent
        raise nodes.SkipNode

    def visit_toctree(self, node: nodes.Element):
        raise nodes.SkipNode

    def visit_index(self, node: nodes.Element):
        raise nodes.SkipNode

    def unknown_visit(self, node: nodes.Node) -> None:
        super().unknown_visit(node)

    def unknown_departure(self, node: nodes.Node) -> None:
        pass
