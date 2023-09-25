"""
Implement https://github.com/syntax-tree/unist.

Upstream: https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/unist/index.d.ts
"""
from __future__ import annotations

from typing import Any, Dict, List, Optional, TypedDict, TypeVar

from typing_extensions import NotRequired


class Point(TypedDict):
    line: int
    column: int
    offset: NotRequired[Optional[int]]


class Position(TypedDict):
    start: Point
    end: Point
    indent: NotRequired[Optional[List[int]]]


class Node(TypedDict):
    type: str
    data: NotRequired[Optional[Dict[str, Any]]]
    position: NotRequired[Optional[Position]]


ChildNode = TypeVar("ChildNode", bound=Node)


class Parent(Node):
    children: List[Node]


class Literal_(Node):
    value: Any
