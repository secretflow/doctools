from __future__ import annotations

from typing import TYPE_CHECKING, Union

from typing_extensions import TypeAlias

if TYPE_CHECKING:
    from . import directive, mdx, unist
    from . import mdast as md

# TODO: pylance is so slow that it is not worth it to fully replicate
# the TypeScript specs (maybe due to PEP 563 and Python<3.10?)

# Content = Union[
#     "TopLevelContent",
#     "ListContent",
#     "TableContent",
#     "RowContent",
#     "PhrasingContent",
# ]

# TopLevelContent = Union[
#     "BlockContent",
#     "FrontmatterContent",
#     "DefinitionContent",
# ]

# BlockContent = Union[
#     "Paragraph",
#     "Heading",
#     "ThematicBreak",
#     "Blockquote",
#     "List_",
#     "Table",
#     "HTML",
#     "Code",
#     "MDXJSXFlowElement",
#     "MDXFlowExpression",
#     "MDXJSESM",
#     "ContainerDirective",
#     "LeafDirective",
# ]

# FrontmatterContent: TypeAlias = "YAML"

# DefinitionContent = Union["Definition", "FootnoteDefinition"]

# ListContent: TypeAlias = "ListItem"

# TableContent: TypeAlias = "TableRow"

# RowContent: TypeAlias = "TableCell"

# PhrasingContent = Union["StaticPhrasingContent", "Link", "LinkReference"]

# StaticPhrasingContent = Union[
#     "Text",
#     "Emphasis",
#     "Strong",
#     "Delete",
#     "HTML",
#     "InlineCode",
#     "Break",
#     "Image",
#     "ImageReference",
#     "Footnote",
#     "FootnoteReference",
#     "MDXJSXTextElement",
#     "MDXTextExpression",
#     "TextDirective",
# ]

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
