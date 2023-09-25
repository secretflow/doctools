"""
Implement MDX-specific ASTs: https://github.com/syntax-tree/mdast-util-mdx

Upstreams:

- https://github.com/syntax-tree/mdast-util-mdx-expression/blob/main/index.d.ts
- https://github.com/syntax-tree/mdast-util-mdx-jsx/blob/main/index.d.ts
- https://github.com/syntax-tree/mdast-util-mdxjs-esm/blob/main/index.d.ts

Note: Currently this doesn't include any of @types/estree.
"""
from __future__ import annotations

import json
from typing import Any, Iterable, List, Literal, Optional, Tuple, Union

from typing_extensions import NotRequired

from . import mdast, unist
from .content_model import BlockContent, DefinitionContent, PhrasingContent


class MDXFlowExpression(mdast.Literal_):
    type: Literal["mdxFlowExpression"]

    # TODO: include estree types
    # https://github.com/syntax-tree/mdast-util-mdx-expression/blob/main/index.d.ts#L50


class MDXTextExpression(mdast.Literal_):
    type: Literal["mdxTextExpression"]


class MDXJSXAttributeValueExpression(mdast.Literal_):
    type: Literal["mdxJsxAttributeValueExpression"]


class MDXJSXExpressionAttribute(mdast.Literal_):
    type: Literal["mdxJsxExpressionAttribute"]


class MDXJSXAttribute(unist.Node):
    type: Literal["mdxJsxAttribute"]
    name: str
    value: NotRequired[Optional[Union[MDXJSXAttributeValueExpression, str]]]


MDXJSXAttributeList = List[Union[MDXJSXAttribute, MDXJSXExpressionAttribute]]


class MDXJSXFlowElement(mdast.Parent):
    type: Literal["mdxJsxFlowElement"]
    name: Union[str, None]
    attributes: MDXJSXAttributeList
    children: List[Union[BlockContent, DefinitionContent]]


class MDXJSXTextElement(mdast.Parent):
    type: Literal["mdxJsxTextElement"]
    name: Union[str, None]
    attributes: MDXJSXAttributeList
    children: List[PhrasingContent]


class MDXJSESM(mdast.Literal_):
    type: Literal["mdxjsEsm"]


def attr_html_like(key: str, value: Union[str, None]) -> MDXJSXAttribute:
    return {
        "type": "mdxJsxAttribute",
        "name": key,
        "value": None if value is None else str(value),
    }


def attr_literal(key: str, expression: Any) -> MDXJSXAttribute:
    return {
        "type": "mdxJsxAttribute",
        "name": key,
        "value": {
            "type": "mdxJsxAttributeValueExpression",
            "value": json.dumps(expression),
        },
    }


def inline_expr(data: Any) -> MDXTextExpression:
    return {
        "type": "mdxTextExpression",
        "value": json.dumps(data),
    }


def set_attribute(
    node: Union[MDXJSXFlowElement, MDXJSXTextElement],
    updated: MDXJSXAttribute,
) -> None:
    for i, attr in enumerate(node["attributes"]):
        if attr.get("name") == updated["name"]:
            node["attributes"][i] = updated
            return
    node["attributes"].append(updated)


def attribute_list(**attributes: Any) -> MDXJSXAttributeList:
    attr_list: MDXJSXAttributeList = []
    for k, v in attributes.items():
        if isinstance(v, str):
            if "\n" in v:
                attr_list.append(attr_literal(k, v))
            else:
                attr_list.append(attr_html_like(k, v))
        elif v is not None:
            attr_list.append(attr_literal(k, v))
    return attr_list


def block(
    __name__: str,
    *,
    classes: Iterable[str] = (),
    **attributes: Any,
) -> MDXJSXFlowElement:
    if classes:
        attributes["className"] = " ".join(classes)
    return {
        "type": "mdxJsxFlowElement",
        "name": __name__,
        "attributes": attribute_list(**attributes),
        "children": [],
    }


def inline(
    __name__: str,
    *,
    classes: Tuple[str, ...] = (),
    **attributes: Any,
) -> MDXJSXTextElement:
    if classes:
        attributes["className"] = " ".join(classes)
    return {
        "type": "mdxJsxTextElement",
        "name": __name__,
        "attributes": attribute_list(**attributes),
        "children": [],
    }


def is_element(node: unist.Node, name: str) -> bool:
    return (
        node["type"] in ("mdxJsxFlowElement", "mdxJsxTextElement")
        and node["name"] == name
    )


def comment(value: str) -> MDXJSXTextElement:
    return {"type": "mdxTextExpression", "value": f"/* {value} */"}
