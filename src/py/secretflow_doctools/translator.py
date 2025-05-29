# pyright: reportArgumentType=false, reportAssignmentType=false

from __future__ import annotations

from collections import ChainMap, defaultdict
from copy import deepcopy
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, DefaultDict, Dict, List, Optional, TypeVar, cast

from docutils import nodes
from loguru import logger
from more_itertools import first
from myst_parser.parsers import parse_html
from sphinx import addnodes
from sphinx.builders import Builder
from sphinx.util.docutils import SphinxTranslator
from typing_extensions import TypeAlias

from secretflow_doctools.i18n.builder import SwaggerGettextBuilder
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.markdown.ffi import MarkdownClient
from secretflow_doctools.markdown.specs import directive, math, mdx
from secretflow_doctools.markdown.specs import mdast as md
from secretflow_doctools.options import MdxConfig
from secretflow_doctools.pathfinding import Pathfinder, StaticFiles

UNSAFE_INLINE_CHARS = "_-*`~[]\n"


@dataclass
class ParentNode:
    origin: nodes.Element
    markup: md.Parent


NodeType = TypeVar("NodeType", bound=nodes.Node)
NodeHandler: TypeAlias = Callable[[NodeType], Optional[bool]]


class MdxTranslator(SphinxTranslator):
    def __init__(
        self,
        document: nodes.document,
        builder: Builder,
        options: MdxConfig,
        mdclient: MarkdownClient,
        pathfinder: Pathfinder,
        static_files: StaticFiles,
        swagger_i18n: SwaggerGettextBuilder,
    ):
        super().__init__(document, builder)

        self.document = document
        self.settings = document.settings

        self.mdclient = mdclient
        self.options = options

        self.root: md.Root
        self.ancestors: List[ParentNode]

        self.metadata: Dict = {}
        self.section_depth = 0
        self.current_line: int = -1

        self.context_info: ChainMap[str, Any] = ChainMap()
        self.context_handlers: DefaultDict[str, List[NodeHandler]] = defaultdict(list)
        self.context_callbacks: Dict[nodes.Node, Callable] = {}

        self.pathfinder = pathfinder

        self.static_files = static_files
        self.swagger_i18n = swagger_i18n

    @property
    def source_path(self) -> Path:
        return Path(self.document["source"])  # pragma: no cover

    @property
    def docname(self) -> str:
        docname = self.builder.env.path2doc(self.document["source"])
        if docname is None:
            raise ValueError()
        return docname

    @property
    def asset_root(self):
        return Path(self.builder.outdir) / self.options.mdx_assets_output_dir

    @property
    def parent(self) -> md.Parent:
        return self.ancestors[-1].markup

    @property
    def is_in_jsx(self):
        return any(
            p.markup["type"] in ("mdxJsxFlowElement", "mdxJsxTextElement")
            for p in self.ancestors
        )

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

    def add_element_id(self, origin: nodes.Element, markup: Optional[md.Content]):
        try:
            primary_id, *secondary_ids = origin["ids"]
        except (ValueError, LookupError):
            return

        if markup is None:
            return

        if markup["type"] in ("mdxJsxFlowElement", "mdxJsxTextElement"):
            # add ID to JSX element
            element = cast(mdx.MDXJSXFlowElement, markup)
            if not any(attr.get("name") == "id" for attr in element["attributes"]):
                element["attributes"].append(mdx.attr_html_like("id", primary_id))
            for secondary_id in secondary_ids:
                element["children"].insert(0, mdx.text("Target", id=secondary_id))
        else:
            # include as a text directive, which could then be processed by remark
            self.parent["children"].append(directive.inline("target", id=primary_id))
            for secondary_id in secondary_ids:
                markup["children"].insert(0, mdx.text("Target", id=secondary_id))

    def append_child(
        self,
        origin: Optional[nodes.Element],
        markup: md.unist.Node,
    ) -> bool:
        try:
            parent = self.parent
        except IndexError:
            return False
        if origin is not None:
            self.add_element_id(origin, markup)
        parent["children"].append(markup)
        return True

    def enter_nested(self, origin: nodes.Element, markup: md.Parent):
        self.ancestors.append(ParentNode(origin, markup))
        return markup

    def leave_nested(self, origin: nodes.Element, *, multiple: bool = False):
        child = self.ancestors.pop()
        if child.origin is not origin:
            # this guarantees parity
            self.ancestors.append(child)
            return child.markup
        self.append_child(origin, child.markup)
        if multiple:
            return self.leave_nested(origin, multiple=True)
        return child.markup

    def visit_Text(self, node: nodes.Text):  # noqa: N802
        self.append_child(None, md.text(node.astext()))

    # === Frontmatter ===

    def visit_meta(self, node: nodes.meta):
        name = node.get("name")
        prop = node.get("property")
        lang = node.get("lang")
        value = node.get("content")
        http_equiv = node.get("http-equiv")

        attributes = {"content": value}

        if name:
            attributes["name"] = name
        elif prop:
            attributes["property"] = prop
        elif http_equiv:
            attributes["httpEquiv"] = http_equiv

        if lang:
            attributes["lang"] = lang

        self.enter_nested(node, mdx.flow("Helmet"))
        self.append_child(node, mdx.flow("meta", **attributes))
        self.leave_nested(node)

        if name:
            # lang support
            self.metadata.setdefault(name, value)

    def visit_document(self, node: nodes.document):
        self.root = md.root()
        self.ancestors = [ParentNode(node, self.root)]
        if node.get("title"):
            self.metadata["title"] = node["title"]

    def depart_document(self, node: nodes.document):
        self.leave_nested(node)
        assert not self.ancestors, (
            f"stack: {self.ancestors}\n"
            "nesting stack is not empty. this indicates"
            " unclosed elements, which is a bug.\n"
            f"file: {self.source_path}"
        )

    def visit_section(self, node: nodes.section):
        self.section_depth += 1

    def depart_section(self, node: nodes.section):
        self.section_depth -= 1

    # === Flow content ===

    def visit_title(self, node: nodes.title):
        if isinstance(node.parent, nodes.table):
            self.enter_nested(node, mdx.flow("caption"))
        elif isinstance(node.parent, nodes.sidebar):
            self.enter_nested(node, mdx.flow("p", classes=("sidebar-title",)))
        elif isinstance(node.parent, addnodes.compact_paragraph):
            self.enter_nested(node, mdx.flow("p", classes=("caption",)))
        elif isinstance(node.parent, (nodes.Admonition, nodes.topic)):
            self.context_info["admonition_title"] = node.astext()
            raise nodes.SkipNode
        else:
            if isinstance(node.parent, nodes.section):
                node["ids"].extend(node.parent["ids"])
            self.enter_nested(node, md.heading(self.section_depth))

    def depart_title(self, node: nodes.title):
        self.leave_nested(node)

    def visit_paragraph(self, node: nodes.paragraph):
        self.enter_nested(node, md.paragraph())

    def depart_paragraph(self, node: nodes.paragraph):
        # multiple: wrap up any potentially unclosed HTML tags
        # handle malformed HTML more gracefully
        self.leave_nested(node, multiple=True)

    swagger_counter = 0

    def visit_literal_block(self, node: nodes.literal_block):
        lang = node.get("language", "plaintext")

        if lang == "ipython3":
            lang = "python"

        # myst-nb
        if lang == "myst-ansi":
            try:
                from ansi2html import Ansi2HTMLConverter

                conv = Ansi2HTMLConverter()
                html = conv.convert(node.astext(), full=False)
                tree = self.mdclient.html_to_tree(html)

            except ValueError:
                pass

            else:
                self.enter_nested(
                    node,
                    mdx.flow("pre", classes=("ansi2html-content",)),
                )
                self.enter_nested(node, mdx.flow("code"))
                for elem in tree["children"]:
                    self.append_child(node, elem)
                self.leave_nested(node)
                self.leave_nested(node)
                raise nodes.SkipNode

        if lang == "swagger":
            msg = self.swagger_i18n.load_messages(self.docname)
            if msg:
                txt = msg.translate(self.swagger_counter, node.astext())
                node.children = [nodes.Text(txt)]
            self.swagger_counter += 1

        content = node.astext().strip()
        self.append_child(node, md.code_block(content, lang))

        raise nodes.SkipNode

    def visit_doctest_block(self, node: nodes.doctest_block):
        node["language"] = "python"
        self.visit_literal_block(node)

    def visit_transition(self, node: nodes.transition):
        self.append_child(node, md.thematic_break())
        raise nodes.SkipNode

    # === Lists ===

    def visit_bullet_list(self, node: nodes.bullet_list):
        self.enter_nested(node, md.unordered_list())

    def depart_bullet_list(self, node: nodes.bullet_list):
        md.List_ = self.leave_nested(node)

    def visit_enumerated_list(self, node: nodes.enumerated_list):
        self.enter_nested(node, md.ordered_list(node.get("start")))

    def depart_enumerated_list(self, node: nodes.enumerated_list):
        md.List_ = self.leave_nested(node)

    def visit_list_item(self, node: nodes.list_item):
        self.enter_nested(node, md.list_item())

    def depart_list_item(self, node: nodes.list_item):
        self.leave_nested(node)

    def visit_definition_list(self, node: nodes.definition_list):
        self.enter_nested(node, mdx.flow("DefinitionList"))
        self.enter_nested(node, mdx.flow("dl"))

    def depart_definition_list(self, node: nodes.definition_list):
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_definition_list_item(self, node: nodes.definition_list_item):
        self.enter_nested(node, mdx.flow("dt"))

    def depart_definition_list_item(self, node: nodes.definition_list_item):
        pass

    def visit_term(self, node: nodes.term):
        self.enter_nested(node, mdx.text("DefinitionList.Term"))
        self.enter_nested(node, mdx.text("span"))

    def depart_term(self, node: nodes.term):
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_classifier(self, node: nodes.classifier):
        self.enter_nested(node, mdx.text("em", classes=("classifier",)))
        self.append_child(node, md.text(" : "))

    def depart_classifier(self, node: nodes.classifier):
        self.leave_nested(node)

    def visit_definition(self, node: nodes.definition):
        self.leave_nested(node.parent)  # close <dt />
        self.enter_nested(node, mdx.flow("dd"))

    def depart_definition(self, node: nodes.definition):
        self.leave_nested(node)

    def visit_field_list(self, node: nodes.field_list):
        self.enter_nested(node, mdx.flow("FieldList"))
        self.enter_nested(node, mdx.flow("dl"))
        self.contextualize(node)

    def depart_field_list(self, node: nodes.field_list):
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_field(self, node: nodes.field):
        # treat as transparent
        pass

    def depart_field(self, node: nodes.field):
        pass

    def visit_field_name(self, node: nodes.field_name):
        self.enter_nested(node, mdx.flow("dt", classes=("field-name",)))
        self.context_info["field_section"] = node.astext()

    def depart_field_name(self, node: nodes.field_name):
        self.leave_nested(node)

    def visit_field_body(self, node: nodes.field_body):
        self.enter_nested(node, mdx.flow("dd", classes=("field-body",)))

    def depart_field_body(self, node: nodes.field_body):
        self.leave_nested(node)

    def visit_option_list(self, node: nodes.option_list):
        self.enter_nested(node, mdx.flow("OptionList"))
        self.enter_nested(node, mdx.flow("dl"))

    def depart_option_list(self, node: nodes.option_list):
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_option_list_item(self, node: nodes.option_list_item):
        pass

    def depart_option_list_item(self, node: nodes.option_list_item):
        pass

    def visit_option_group(self, node: nodes.option_group):
        self.enter_nested(node, mdx.flow("dt"))

    def depart_option_group(self, node: nodes.option_group):
        self.leave_nested(node)

    def visit_option(self, node: nodes.option):
        if node.parent.index(node) > 0:
            self.append_child(node, md.text(", "))
        self.enter_nested(node, mdx.text("span", classes=("option",)))

    def depart_option(self, node: nodes.option):
        self.leave_nested(node)

    def visit_option_string(self, node: nodes.option_string):
        self.enter_nested(node, mdx.text("span"))
        self.append_child(node, md.text(node.astext()))
        self.leave_nested(node)
        raise nodes.SkipNode

    def visit_option_argument(self, node: nodes.option_argument):
        self.enter_nested(node, mdx.text("span", classes=("argument",)))
        self.append_child(node, md.text(node.astext()))
        self.leave_nested(node)
        raise nodes.SkipNode

    def visit_description(self, node: nodes.description):
        self.enter_nested(node, mdx.flow("dd"))

    def depart_description(self, node: nodes.description):
        self.leave_nested(node)

    # === Blockquotes ===

    def visit_block_quote(self, node: nodes.block_quote):
        self.enter_nested(node, md.blockquote())

    def depart_block_quote(self, node: nodes.block_quote):
        self.leave_nested(node)

    def visit_line_block(self, node: nodes.line_block):
        self.enter_nested(node, mdx.flow("LineBlock"))

    def depart_line_block(self, node: nodes.line_block):
        self.leave_nested(node)

    def visit_line(self, node: nodes.line):
        if not node.children:
            self.append_child(node, mdx.text("br"))
            raise nodes.SkipNode

        self.enter_nested(node, md.paragraph())

    def depart_line(self, node: nodes.line):
        self.leave_nested(node)

    def visit_attribution(self, node: nodes.attribution):
        self.enter_nested(node, mdx.text("em", classes=("attribution",)))
        self.append_child(node, md.text("-- "))

    def depart_attribution(self, node: nodes.attribution):
        self.leave_nested(node)

    # === Figures ===

    def visit_figure(self, node: nodes.figure):
        self.enter_nested(node, mdx.flow("figure"))

    def depart_figure(self, node: nodes.figure):
        if mdx.is_element(self.parent, "figcaption"):
            caption = node.next_node(nodes.caption)
            self.leave_nested(caption)
        self.leave_nested(node)

    def visit_caption(self, node: nodes.caption):
        self.enter_nested(node, mdx.flow("figcaption"))

    def depart_caption(self, node: nodes.caption):
        self.leave_nested(node)

    def visit_legend(self, node: nodes.legend):
        pass

    def depart_legend(self, node: nodes.legend):
        pass

    # === Inline content ===

    def visit_inline(self, node: nodes.inline):
        if self.is_in_jsx:
            self.enter_nested(node, mdx.text("span"))

    def depart_inline(self, node: nodes.inline):
        self.leave_nested(node)

    def visit_strong(self, node: nodes.strong):
        self.enter_nested(node, mdx.text("strong"))

    def depart_strong(self, node: nodes.strong):
        self.leave_nested(node)

    def visit_emphasis(self, node: nodes.emphasis):
        self.enter_nested(node, mdx.text("em"))

    def depart_emphasis(self, node: nodes.emphasis):
        self.leave_nested(node)

    def visit_literal(self, node: nodes.literal):
        classes = node.get("classes", [])
        if node.get("language"):
            # inline code with language support
            elem = mdx.text("code", **{"data-language": node.get("language")})
            self.enter_nested(node, elem)
            self.append_child(node, md.text(node.astext()))
            self.leave_nested(node)
        elif "kbd" in classes:
            # keyboard input
            self.enter_nested(node, mdx.text("kbd"))
            self.append_child(node, md.text(node.astext()))
            self.leave_nested(node)
        elif "download" in classes:
            # download link (already wrapped in <a />)
            self.append_child(node, md.text(node.astext()))
        else:
            self.append_child(node, md.inline_code(node.astext()))
        raise nodes.SkipNode

    def visit_subscript(self, node: nodes.subscript):
        self.enter_nested(node, mdx.text("sub"))

    def depart_subscript(self, node: nodes.subscript):
        self.leave_nested(node)

    def visit_superscript(self, node: nodes.superscript):
        self.enter_nested(node, mdx.text("sup"))

    def depart_superscript(self, node: nodes.superscript):
        self.leave_nested(node)

    # === References ===

    def visit_reference(self, node: nodes.reference):
        # From HTML5Translator
        url: str

        if "refid" in node:
            url = f"#{node['refid']}"

        elif "refuri" in node:
            refuri = node["refuri"]

            if not refuri:
                # empty, probably invalid
                url = "#"

            if self.pathfinder.is_external_url(refuri):
                url = refuri

            else:
                url = self.pathfinder.get_output_path_from_refuri(refuri)

        else:
            return

        self.enter_nested(node, md.link(url, node.get("reftitle")))

    def depart_reference(self, node: nodes.reference):
        self.leave_nested(node)

    def visit_footnote_reference(self, node: nodes.footnote_reference):
        self.enter_nested(node, mdx.text("sup"))
        self.visit_reference(node)

    def depart_footnote_reference(self, node: nodes.footnote_reference):
        self.depart_reference(node)
        self.leave_nested(node)

    def visit_footnote(self, node: nodes.footnote):
        label_node = node.next_node(nodes.label)
        if label_node:
            label = label_node.astext()
        else:
            label = None
        self.enter_nested(
            node,
            mdx.flow("Footnote", label=label, backrefs=node.get("backrefs")),
        )

    def depart_footnote(self, node: nodes.footnote):
        self.leave_nested(node)

    def visit_target(self, node: nodes.Element):
        if not len(node):
            return
        self.enter_nested(node, mdx.text("span"))

    def depart_target(self, node: nodes.Element):
        self.leave_nested(node)

    def visit_citation(self, node: nodes.citation):
        self.visit_footnote(node)

    def depart_citation(self, node: nodes.citation):
        self.depart_footnote(node)

    def visit_label(self, node: nodes.label):
        raise nodes.SkipNode

    def visit_image(self, node: nodes.image):
        src = self.static_files.add_image(self.docname, node)
        title = node.get("title", None)
        alt = node.get("alt", None)
        image = md.image(src, title=title, alt=alt)
        self.append_child(node, image)
        raise nodes.SkipNode

    def visit_download_reference(self, node: addnodes.download_reference):
        uri = self.static_files.add_downloadable_file(self.docname, node)
        self.enter_nested(node, mdx.text("a", href=uri, download=True))

    def depart_download_reference(self, node: addnodes.download_reference):
        self.leave_nested(node)

    # === Sphinx: standard admonitions ===

    def visit_Admonition(self, node: nodes.Admonition):  # noqa: N802
        name = type(node).__name__
        title = node.get("title")  # type: ignore
        # We were supposed to use remark-directives here, but there are parsing issues
        # when nested inside a JSX component "Expected the closing tag `...` either
        # after the end of `directiveContainer` ...". Further investigation is needed.
        self.enter_nested(node, mdx.flow("Container", type=name, title=title))
        self.contextualize(node)

    def depart_Admonition(self, node: nodes.Admonition):  # noqa: N802
        container: mdx.MDXJSXFlowElement = self.leave_nested(node)
        if title := self.context_info.get("admonition_title"):
            mdx.set_attribute(container, mdx.attr_literal("title", title))

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
        self.enter_nested(node, mdx.flow("Container", type=name, title=title))

    def depart_versionmodified(self, node: addnodes.versionmodified):
        self.leave_nested(node)

    def visit_seealso(self, node: addnodes.seealso):
        self.enter_nested(node, mdx.flow("Container", type="info", title="See also:"))

    def depart_seealso(self, node: addnodes.seealso):
        self.leave_nested(node)

    # === Sphinx: Horizontal lists ===

    def visit_hlist(self, node: addnodes.hlist) -> None:
        self.enter_nested(node, mdx.flow("HorizontalList"))
        self.enter_nested(node, mdx.flow("table"))
        self.enter_nested(node, mdx.flow("tr"))

    def depart_hlist(self, node: addnodes.hlist) -> None:
        self.leave_nested(node)
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_hlistcol(self, node: addnodes.hlistcol) -> None:
        self.enter_nested(node, mdx.flow("td"))

    def depart_hlistcol(self, node: addnodes.hlistcol) -> None:
        self.leave_nested(node)

    # === Sphinx: symbol/signature ===

    def visit_desc(self, node: addnodes.desc):
        domain: Optional[str] = node.attributes.get("domain")
        obj_type: Optional[str] = node.attributes.get("desctype")

        elem = mdx.flow("Outline", domain=domain, objectType=obj_type)
        self.enter_nested(node, elem)
        self.contextualize(node)

    def depart_desc(self, node: addnodes.desc):
        element: mdx.MDXJSXFlowElement = self.leave_nested(node)
        info = {**self.context_info}
        for k, v in info.items():
            element["attributes"].append(mdx.attr_literal(k, v))

    def visit_desc_signature(self, node: addnodes.desc_signature):
        elem = mdx.flow("Outline.Signature", fullname=node.get("fullname"))
        self.enter_nested(node, elem)

        self.context_info["target"] = first(node.get("ids"), None)
        self.context_info["module"] = node.get("module")
        self.context_info["fullname"] = node.get("fullname")

    def depart_desc_signature(self, node: addnodes.desc_signature):
        self.leave_nested(node)

    def visit_desc_name(self, node: addnodes.desc_name):
        self.enter_nested(node, mdx.text("Outline.Name"))

    def depart_desc_name(self, node: addnodes.desc_name):
        self.leave_nested(node)

    def visit_desc_addname(self, node: addnodes.desc_addname):
        self.enter_nested(node, mdx.text("Outline.Prefix"))

    def depart_desc_addname(self, node: addnodes.desc_addname):
        self.leave_nested(node)

    def visit_desc_annotation(self, node: addnodes.desc_annotation):
        self.enter_nested(node, mdx.text("Outline.Keyword"))

    def depart_desc_annotation(self, node: addnodes.desc_annotation):
        self.leave_nested(node)

    def visit_desc_signature_line(self, node: addnodes.desc_signature_line):
        self.enter_nested(node, md.paragraph())

    def depart_desc_signature_line(self, node: addnodes.desc_signature_line):
        self.leave_nested(node)

    first_param: int
    optional_param_level: int
    required_params_left: int
    param_separator: str

    def visit_desc_parameterlist(self, node: addnodes.desc_parameterlist):
        self.enter_nested(node, mdx.text("Outline.ParameterList"))
        self.enter_nested(node, mdx.text("span"))
        self.append_child(node, md.text("("))
        self.leave_nested(node)

        # from sphinx.writers.html5.HTML5Translator
        self.first_param = 1
        self.optional_param_level = 0
        self.required_params_left = sum(
            [isinstance(c, addnodes.desc_parameter) for c in node.children]
        )
        self.param_separator = node.child_text_separator

        self.context_info["parameters"] = []

    def depart_desc_parameterlist(self, node: addnodes.desc_parameterlist):
        self.enter_nested(node, mdx.text("span"))
        self.append_child(node, md.text(")"))
        self.leave_nested(node)
        self.leave_nested(node)

    def visit_desc_parameter(self, node: addnodes.desc_parameter):
        if self.first_param:
            self.first_param = 0
        elif not self.required_params_left:
            self.append_child(node, md.text(self.param_separator))
        if self.optional_param_level == 0:
            self.required_params_left -= 1

        self.enter_nested(node, mdx.text("Outline.Parameter"))

        name = node.next_node(addnodes.desc_sig_name)
        if name is not None:
            self.context_info["parameters"].append(name.astext())

    def depart_desc_parameter(self, node: addnodes.desc_parameter):
        self.leave_nested(node)
        if self.required_params_left:
            self.append_child(node, md.text(self.param_separator))

    def visit_desc_optional(self, node: addnodes.desc_optional) -> None:
        self.optional_param_level += 1
        self.append_child(node, md.text("["))

    def depart_desc_optional(self, node: addnodes.desc_optional) -> None:
        self.optional_param_level -= 1
        self.append_child(node, md.text("]"))

    def visit_desc_sig_name(self, node: addnodes.desc_sig_name):
        self.enter_nested(node, mdx.text("span", classes=("name",)))

    def depart_desc_sig_name(self, node: addnodes.desc_sig_name):
        self.leave_nested(node)

    def visit_desc_sig_space(self, node: addnodes.desc_sig_space):
        self.enter_nested(node, mdx.text("span"))
        self.append_child(node, mdx.inline_expr(" "))
        self.leave_nested(node)
        raise nodes.SkipNode

    def visit_desc_sig_punctuation(self, node: addnodes.desc_sig_punctuation):
        self.enter_nested(node, mdx.text("span"))

    def depart_desc_sig_punctuation(self, node: addnodes.desc_sig_punctuation):
        self.leave_nested(node)

    def visit_desc_sig_operator(self, node: addnodes.desc_sig_operator):
        self.enter_nested(node, mdx.text("span"))

    def depart_desc_sig_operator(self, node: addnodes.desc_sig_operator):
        self.leave_nested(node)

    def visit_desc_inline(self, node: addnodes.desc_inline):
        self.enter_nested(node, mdx.text("code"))

    def depart_desc_inline(self, node: addnodes.desc_inline):
        self.leave_nested(node)

    def visit_desc_type(self, node: addnodes.desc_type):
        self.enter_nested(node, mdx.text("Outline.TypeAnnotation"))

    def depart_desc_type(self, node: addnodes.desc_type):
        self.leave_nested(node)

    def visit_desc_returns(self, node: addnodes.desc_returns):
        self.enter_nested(node, mdx.text("Outline.ReturnType"))
        self.enter_nested(node, mdx.text("span"))
        self.append_child(node, mdx.inline_expr(" → "))
        self.leave_nested(node)

    def depart_desc_returns(self, node: addnodes.desc_returns):
        self.leave_nested(node)

    def visit_desc_content(self, node: addnodes.desc_content):
        self.enter_nested(node, mdx.flow("Outline.Content"))

        description = node.next_node(nodes.paragraph)
        if description:
            self.context_info["description"] = description.astext()

        def visit_literal_strong(node: addnodes.literal_strong):
            if self.context_info.get("field_section") != "Parameters":
                return False
            self.enter_nested(node, mdx.text("Outline.ParameterTarget"))

        self.contextualize(node, visit_literal_strong)

    def depart_desc_content(self, node: addnodes.desc_content):
        self.leave_nested(node)

    # === Tables ===

    def visit_table(self, node: nodes.table):
        self.enter_nested(node, mdx.flow("table"))

    def depart_table(self, node: nodes.table):
        self.leave_nested(node)

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
        self.enter_nested(node, mdx.flow("thead"))

    def depart_thead(self, node: nodes.thead):
        self.leave_nested(node)

    def visit_tbody(self, node: nodes.tbody):
        self.enter_nested(node, mdx.flow("tbody"))

    def depart_tbody(self, node: nodes.tbody):
        self.leave_nested(node)

    def visit_row(self, node: nodes.row):
        attrs: Dict[str, Any] = {}
        if node.get("morecols"):
            attrs["colSpan"] = int(node["morecols"]) + 1
        if node.get("morerows"):
            attrs["rowSpan"] = int(node["morerows"]) + 1
        self.enter_nested(node, mdx.flow("tr", **attrs))

    def depart_row(self, node: nodes.row):
        self.leave_nested(node)

    def visit_entry(self, node: nodes.entry):
        attrs: Dict[str, Any] = {}
        if node.get("morecols"):
            attrs["colSpan"] = int(node["morecols"]) + 1
        if node.get("morerows"):
            attrs["rowSpan"] = int(node["morerows"]) + 1
        self.enter_nested(node, mdx.flow("td", **attrs))

    def depart_entry(self, node: nodes.entry):
        self.leave_nested(node)

    # === Sidebar ===

    def visit_sidebar(self, node: nodes.sidebar):
        self.enter_nested(node, mdx.flow("aside"))

    def depart_sidebar(self, node: nodes.sidebar):
        self.leave_nested(node)

    def visit_subtitle(self, node: nodes.subtitle):
        self.enter_nested(node, mdx.flow("p", classes=("sidebar-subtitle",)))

    def depart_subtitle(self, node: nodes.subtitle):
        self.leave_nested(node)

    def visit_rubric(self, node: nodes.rubric):
        self.enter_nested(node, mdx.flow("p", classes=("sidebar-rubric",)))

    def depart_rubric(self, node: nodes.rubric):
        self.leave_nested(node)

    # === Semantic text markup ===

    def visit_title_reference(self, node: nodes.title_reference):
        self.enter_nested(node, mdx.text("cite"))

    def depart_title_reference(self, node: nodes.title_reference):
        self.leave_nested(node)

    def visit_abbreviation(self, node: nodes.abbreviation):
        self.enter_nested(node, mdx.text("abbr"))

    def depart_abbreviation(self, node: nodes.abbreviation):
        self.leave_nested(node)

    def visit_acronym(self, node: nodes.acronym):
        self.visit_abbreviation(node)

    def depart_acronym(self, node: nodes.acronym):
        self.depart_abbreviation(node)

    # === Semantic containers ===

    def visit_compound(self, node: nodes.compound):
        if "toctree-wrapper" in node.get("classes", ()):
            self.enter_nested(node, mdx.flow("TableOfContents"))

    def depart_compound(self, node: nodes.compound):
        self.leave_nested(node)

    def visit_glossary(self, node: addnodes.glossary):
        pass

    def depart_glossary(self, node: addnodes.glossary):
        pass

    # === Math ===

    def visit_math(self, node: nodes.math):
        self.enter_nested(node, mdx.text("InlineMath"))
        self.append_child(node, math.inline_math(node.astext()))
        self.leave_nested(node)
        raise nodes.SkipNode

    def visit_math_block(self, node: nodes.math_block):
        for chunk in node.astext().split("\n\n"):
            self.enter_nested(node, mdx.flow("Math"))
            self.append_child(node, math.math(chunk.strip()))
            self.leave_nested(node)
        raise nodes.SkipNode

    # === Raw ===

    def visit_raw(self, node: nodes.raw):
        if node.get("format") != "html":
            raise nodes.SkipNode

        def maybe_html(node: nodes.Element):
            try:
                return self.mdclient.html_to_tree(node.astext())
            except ValueError as e:
                logger.error(_("malformed HTML: %s"), node.astext(), location=node)
                raise nodes.SkipNode from e

        if isinstance(node.parent, nodes.paragraph):
            # We are inline
            # opening and closing tags are separated into
            # two different raw nodes
            try:
                tag = next(parse_html.tokenize_html(node.astext()).walk())

            except StopIteration:
                # empty AST, assume to be a closing tag
                self.leave_nested(node.parent)

            else:
                transformed = maybe_html(node)

                # XHTML tags are considered "startendtags" in the parser
                if isinstance(tag, (parse_html.VoidTag, parse_html.XTag)):
                    for elem in transformed["children"]:
                        self.append_child(node.parent, elem)
                        break

                elif isinstance(tag, parse_html.Tag):
                    for elem in transformed["children"]:
                        self.enter_nested(node.parent, elem)
                        break

        else:
            # We are a block element, treat the entire node as HTML
            transformed = maybe_html(node)
            for elem in transformed["children"]:
                self.append_child(node, elem)

        raise nodes.SkipNode

    # === Extension: autosummary ===

    def visit_autosummary_table(self, node: nodes.comment):
        pass
        # raise nodes.SkipNode

    def depart_autosummary_table(self, node: nodes.comment):
        pass

    # === Extension: sphinx.ext.graphviz ===

    def visit_graphviz(self, node: nodes.Element):
        self.append_child(node, mdx.flow("Graphviz", code=node["code"]))
        raise nodes.SkipNode

    # === Extension: sphinxcontrib-mermaid ===

    def visit_mermaid(self, node: nodes.Element):
        self.append_child(
            node,
            mdx.flow("Mermaid", code=node["code"], align=node.get("align")),
        )
        raise nodes.SkipNode

    # === Wildcard containers ===

    def visit_container(self, node: nodes.container):
        # Sphinx design

        component_type = node.get("design_component")

        if component_type:
            component = mdx.flow("SphinxDesign", type=component_type)
            self.enter_nested(node, component)

            if component_type == "card":

                def visit_reference(node: nodes.reference):
                    if "sd-stretched-link" not in node["classes"]:
                        return False
                    href = mdx.attr_html_like("href", node["refuri"])
                    component["attributes"].append(href)
                    raise nodes.SkipNode

                def depart_reference(node: nodes.reference):
                    pass

                self.contextualize(node, visit_reference, depart_reference)

            return

        # myst-nb

        nb_element = node.get("nb_element")

        if nb_element:
            if nb_element == "cell_code":
                self.context_info["cell_index"] = node.get("cell_index")
                self.enter_nested(node, mdx.flow("Notebook.Cell"))
                return

            if nb_element == "cell_code_source":
                self.enter_nested(
                    node,
                    mdx.flow(
                        "Notebook.CodeArea",
                        prompt=f"[{self.context_info['cell_index']}]:",
                        stderr=False,
                        type="input",
                    ),
                )
                return

            if nb_element == "cell_code_output":
                prompt = f"[{self.context_info['cell_index']}]:"

                if len(node.children) == 1 and (block := node.children[0]):
                    if isinstance(block, nodes.literal_block):
                        stderr = "stderr" in block.get("classes", [])
                        component = "Notebook.CodeArea"
                    else:
                        stderr = False
                        component = "Notebook.FancyOutput"
                else:
                    stderr = False
                    component = "Notebook.FancyOutput"

                if node.children:
                    self.enter_nested(
                        node,
                        mdx.flow(
                            component,
                            prompt=prompt,
                            stderr=stderr,
                            type="output",
                        ),
                    )

                return

        if node.next_node(nodes.caption, descend=True, siblings=False):
            self.enter_nested(node, mdx.flow("figure"))
            return

    def depart_container(self, node: nodes.container):
        self.leave_nested(node)

    # === Other ===

    def visit_PassthroughTextElement(self, node: nodes.Element):  # noqa: N802
        pass

    def depart_PassthroughTextElement(self, node: nodes.Element):  # noqa: N802
        pass

    def visit_comment(self, node: nodes.comment):
        raise nodes.SkipNode

    def visit_topic(self, node: nodes.topic):
        self.enter_nested(node, mdx.flow("Container", type="info"))

    def depart_topic(self, node: nodes.topic):
        container: mdx.MDXJSXFlowElement = self.leave_nested(node)
        if title := self.context_info.get("admonition_title"):
            mdx.set_attribute(container, mdx.attr_literal("title", title))

    def visit_problematic(self, node: nodes.problematic):
        self.enter_nested(node, mdx.text("del"))

    def depart_problematic(self, node: nodes.problematic):
        self.leave_nested(node)

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

    def export(self) -> str:
        root = deepcopy(self.root)
        if self.metadata:
            root["children"].insert(0, md.frontmatter(self.metadata))
        return self.mdclient.tree_to_markdown(root)
