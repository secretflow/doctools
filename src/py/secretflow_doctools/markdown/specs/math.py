from __future__ import annotations

from typing import Literal

from . import mdast


class Math(mdast.Literal_):
    type: Literal["math"]
    value: str


class InlineMath(mdast.Literal_):
    type: Literal["inlineMath"]
    value: str


def math(value: str) -> Math:
    return {"type": "math", "value": value}


def inline_math(value: str) -> InlineMath:
    return {"type": "inlineMath", "value": value}
