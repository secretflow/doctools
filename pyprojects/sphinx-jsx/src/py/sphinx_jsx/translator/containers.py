from typing import Optional

from docutils import nodes
from sphinx import addnodes

from sphinx_jsx.syntax import rst
from sphinx_jsx.syntax.jsx.models import (
    JSON_NULL,
    JSFragment,
    JSXElement,
    PseudoElement,
)

from .base import BaseJSXTranslator
from .pseudo import Pseudo


class Heading(JSXElement):
    depth: int

    class Rubric(JSXElement):
        pass


class Containers(PseudoElement):
    class Section(JSXElement):
        pass

    class Generic(JSXElement):
        pass

    class Transition(JSXElement):
        pass

    class CodeBlock(JSXElement):
        content: str
        language: Optional[str] = None
        doctest: bool = False
        caption: JSFragment = JSON_NULL

    class Blockquote(JSXElement):
        class Attribution(JSXElement):
            pass

    class LineBlock(JSXElement):
        class Line(JSXElement):
            pass

    class Topic(JSXElement):
        title: JSFragment = JSON_NULL

    class Sidebar(JSXElement):
        title: JSFragment = JSON_NULL
        subtitle: JSFragment = JSON_NULL

    class Outline(JSXElement):
        # .. contents ::
        # i.e. table of contents for current document
        title: JSFragment = JSON_NULL

    class RelatedDocs(JSXElement):
        # .. toctree ::
        title: JSFragment = JSON_NULL

    class Admonition(JSXElement):
        level: str = "admonition"
        title: JSFragment = JSON_NULL

    class VersionModified(JSXElement):
        type: str
        version: str

    class SeeAlso(JSXElement):
        pass

    class Centered(JSXElement):
        pass

    class CompoundParagraph(JSXElement):
        pass


class ContainerMarkupTranslator(BaseJSXTranslator):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    @property
    def section_depth(self):
        return sum(isinstance(e, Containers.Section) for e in self.ancestors)

    def visit_container(self, node: nodes.container):
        self.enter_nesting(node, Containers.Generic())

    def visit_section(self, node: nodes.section):
        self.enter_nesting(node, Containers.Section())

    def depart_section(self, node: nodes.section):
        self.leave_nesting(node, Containers.Section)

    def visit_title(self, node: nodes.title):
        self.enter_nesting(node, Pseudo._Title())

    def visit_subtitle(self, node: nodes.subtitle):
        self.enter_nesting(node, Pseudo._Subtitle())

    def visit_title__heading(self, node: rst.e.title__heading):
        self.enter_nesting(node, Heading(depth=self.section_depth))

    def visit_transition(self, node: nodes.transition):
        self.append_child(node, Containers.Transition())

    def visit_literal_block(self, node: nodes.literal_block):
        lang = node.get("language", "plaintext")
        content = node.astext().strip()
        self.append_child(node, Containers.CodeBlock(content=content, language=lang))
        raise nodes.SkipNode

    def visit_line_block(self, node: nodes.line_block):
        self.enter_nesting(node, Containers.LineBlock())

    def visit_line(self, node: nodes.line):
        self.enter_nesting(node, Containers.LineBlock.Line())

    def visit_block_quote(self, node: nodes.block_quote):
        self.enter_nesting(node, Containers.Blockquote())

    def visit_attribution(self, node: nodes.attribution):
        self.enter_nesting(node, Containers.Blockquote.Attribution())

    def visit_doctest_block(self, node: nodes.doctest_block):
        lang = "python"
        content = node.astext().strip()
        self.append_child(node, Containers.CodeBlock(content=content, language=lang))
        raise nodes.SkipNode

    def visit_container__code(self, node: rst.e.container__code):
        self.enter_nesting(node, Containers.Generic())

    def depart_container__code(self, node: rst.e.container__code):
        div = self.leave_nesting(node, Containers.Generic)
        code_block = div.select_child(Containers.CodeBlock)
        code_block.caption = div.remove_fragment(Pseudo._Caption)

    def visit_sidebar(self, node: nodes.sidebar):
        self.enter_nesting(node, Containers.Sidebar())

    def depart_sidebar(self, node: nodes.sidebar):
        sidebar = self.leave_nesting(node, Containers.Sidebar)
        sidebar.title = sidebar.remove_fragment(Pseudo._Title)
        sidebar.subtitle = sidebar.remove_fragment(Pseudo._Subtitle)

    def visit_centered(self, node: addnodes.centered):
        self.enter_nesting(node, Containers.Centered())

    def visit_Admonition(self, node: nodes.Element):
        self.enter_nesting(node, Containers.Admonition(level=type(node).__name__))

    def depart_Admonition(self, node: nodes.Element):
        elem = self.leave_nesting(node, Containers.Admonition)
        elem.title = elem.remove_fragment(Pseudo._Title)

    def visit_topic(self, node: nodes.topic):
        self.enter_nesting(node, Containers.Topic())

    def depart_topic(self, node: nodes.topic):
        elem = self.leave_nesting(node, Containers.Topic)
        elem.title = elem.remove_fragment(Pseudo._Title)

    def visit_topic__outline(self, node: rst.e.topic__outline):
        self.enter_nesting(node, Containers.Outline())

    def depart_topic__outline(self, node: rst.e.topic__outline):
        elem = self.leave_nesting(node, Containers.Outline)
        elem.title = elem.remove_fragment(Pseudo._Title)

    def visit_compact_paragraph(self, node: addnodes.compact_paragraph):
        if node.get("toctree"):
            # toctree is special in that it isn't touched by post transforms
            # hence we can't process it in TypePromotionTransform
            self.enter_nesting(node, Containers.RelatedDocs())
        else:
            self.enter_nesting(node, Containers.Generic())

    def depart_compact_paragraph(self, node: addnodes.compact_paragraph):
        elem = self.leave_nesting(node)
        if isinstance(elem, Containers.RelatedDocs):
            elem.title = elem.remove_fragment(Pseudo._Title)

    def visit_rubric(self, node: nodes.rubric):
        self.enter_nesting(node, Heading.Rubric())

    def visit_versionmodified(self, node: addnodes.versionmodified):
        self.enter_nesting(
            node,
            Containers.VersionModified(type=node["type"], version=node["version"]),
        )

    def depart_versionmodified(self, node: addnodes.versionmodified):
        self.leave_nesting(node)

    def visit_seealso(self, node: addnodes.seealso):
        self.enter_nesting(node, Containers.SeeAlso())

    def depart_seealso(self, node: addnodes.seealso):
        self.leave_nesting(node)

    def visit_compound(self, node: nodes.compound):
        self.enter_nesting(node, Containers.CompoundParagraph())

    def visit_glossary(self, node: addnodes.glossary):
        pass
