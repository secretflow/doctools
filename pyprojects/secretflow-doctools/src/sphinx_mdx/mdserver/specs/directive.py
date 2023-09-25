"""
Implement https://github.com/syntax-tree/mdast-util-directive.

Upstream: https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/mdast/index.d.ts
"""

from __future__ import annotations

from typing import Dict, List, Literal, Optional, TypedDict, Union

from typing_extensions import NotRequired

from . import mdast
from .content_model import BlockContent, DefinitionContent


class DirectiveFields(TypedDict):
    name: str
    attributes: NotRequired[Optional[Dict[str, Optional[str]]]]


class ContainerDirective(mdast.Parent, DirectiveFields):
    type: Literal["containerDirective"]
    children: List[Union[BlockContent, DefinitionContent]]


class LeafDirective(mdast.Parent, DirectiveFields):
    type: Literal["leafDirective"]
    children: List[mdast.PhrasingContent]


class TextDirective(mdast.Parent, DirectiveFields):
    type: Literal["textDirective"]
    children: List[mdast.PhrasingContent]


Directive = Union[ContainerDirective, LeafDirective, TextDirective]


def inline(name: str, **attributes: str) -> TextDirective:
    return {
        "type": "textDirective",
        "name": name,
        "attributes": {**attributes},
        "children": [],
    }


def container(
    name: str,
    *,
    label: Optional[str] = None,
    **attributes: str,
):
    container: ContainerDirective = {
        "type": "containerDirective",
        "attributes": {**attributes},
        "name": name,
        "children": [],
    }
    if label is not None:
        container["children"].append(
            {
                "type": "paragraph",
                "data": {"directiveLabel": True},
                "children": [{"type": "text", "value": label}],
            }
        )
    return container
