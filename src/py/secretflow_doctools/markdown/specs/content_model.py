from __future__ import annotations

from typing import TYPE_CHECKING, Union

from typing_extensions import TypeAlias

if TYPE_CHECKING:
    from . import directive, mdx, unist
    from . import mdast as md

Content: TypeAlias = "unist.Parent"

TopLevelContent: TypeAlias = "unist.Parent"

BlockContent: TypeAlias = "unist.Parent"

FrontmatterContent: TypeAlias = "unist.Parent"

DefinitionContent: TypeAlias = "unist.Parent"

ListContent: TypeAlias = "unist.Parent"

TableContent: TypeAlias = "unist.Parent"

RowContent: TypeAlias = "unist.Parent"

PhrasingContent: TypeAlias = "unist.Parent"

StaticPhrasingContent: TypeAlias = "unist.Parent"

_Nodes = Union[
    "md.HTML",
    "md.YAML",
    "md.Blockquote",
    "md.Break",
    "md.Code",
    "md.Definition",
    "md.Delete",
    "md.Emphasis",
    # "md.Footnote",
    "md.FootnoteDefinition",
    "md.FootnoteReference",
    "md.Heading",
    "md.Image",
    "md.ImageReference",
    "md.InlineCode",
    "md.Link",
    "md.LinkReference",
    "md.ListItem",
    "md.List_",
    "md.Paragraph",
    "md.Strong",
    "md.Table",
    "md.TableCell",
    "md.TableRow",
    "md.Text",
    "md.ThematicBreak",
    "directive.ContainerDirective",
    "directive.LeafDirective",
    "directive.TextDirective",
    "mdx.MDXFlowExpression",
    "mdx.MDXJSESM",
    "mdx.MDXJSXAttribute",
    "mdx.MDXJSXAttributeValueExpression",
    "mdx.MDXJSXExpressionAttribute",
    "mdx.MDXJSXFlowElement",
    "mdx.MDXJSXTextElement",
    "mdx.MDXTextExpression",
]
