from typing import Optional

import pytest

from sphinx_jsx.syntax.jsx.models import (
    JSObject,
    JSONExpression,
    JSXElement,
    PseudoElement,
    UnsafeInline,
    json_with_jsx,
)


class Element(JSXElement, tag_name="Element"):
    class Subelement(JSXElement, tag_name="Subelement"):
        pass

    class _FakeElement(PseudoElement):
        pass


class Data(JSObject):
    x: int


class Custom:
    pass


class ElementWithProps(JSXElement):
    x: int
    y: float
    z: JSXElement
    w: Optional[bool]
    u: UnsafeInline


def test_json_with_jsx():
    def default(x):
        if isinstance(x, Custom):
            return "custom"
        raise TypeError

    value = {
        "elem": Element(),
        "data": Data(x=42),
        "expr": UnsafeInline(value="{...props}"),
        "custom": Custom(),
    }

    assert json_with_jsx(value, default=default) == (
        "{"
        '"elem":<Element></Element>,'
        '"data":{"x":42},'
        '"expr":{...props},'
        '"custom":"custom"'
        "}"
    )

    with pytest.raises(TypeError):
        json_with_jsx({"custom": Custom()})


def test_json_expression():
    null = JSONExpression(value=None)
    number = JSONExpression(value=42)
    string = JSONExpression(value="string")
    boolean = JSONExpression(value=True)
    nan = JSONExpression(value=float("nan"))
    inf = JSONExpression(value=float("inf"))
    array = JSONExpression(value=[None, 42, "string", True])
    object_ = JSONExpression(value={"lorem": {"ipsum": ["dolor"]}})

    assert null.render() == "null"
    assert number.render() == "42"
    assert string.render() == '"string"'
    assert boolean.render() == "true"

    assert nan.render() == "null"
    assert inf.render() == "null"

    assert array.render() == '[null,42,"string",true]'
    assert object_.render() == '{"lorem":{"ipsum":["dolor"]}}'


def test_props():
    elem = ElementWithProps(
        ids=["id1", "id2"],
        classnames=["cls-1", "cls-2"],
        x=42,
        y=3.14,
        z=Element(),
        w=None,
        u=UnsafeInline(value='new URL(".", import.meta.url)'),
    )
    elem._attrs = {"aria-label": "label", "style": "color: red;"}

    props = elem.props()

    assert next(props).render() == 'className={"cls-1 cls-2"}'
    assert next(props).render() == 'id={"id1"}'
    assert next(props).render() == 'extraIds={["id2"]}'

    assert next(props).render() == 'aria-label={"label"}'

    assert next(props).render() == "x={42}"
    assert next(props).render() == "y={3.14}"
    assert next(props).render() == "z={<Element></Element>}"
    assert next(props).render() == 'u={new URL(".", import.meta.url)}'

    with pytest.raises(StopIteration):
        next(props)
