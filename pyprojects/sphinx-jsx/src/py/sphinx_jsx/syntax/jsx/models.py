from __future__ import annotations

from abc import ABC, abstractmethod
from itertools import chain
from typing import (
    Any,
    Callable,
    ClassVar,
    Dict,
    Iterator,
    List,
    Literal,
    Optional,
    Type,
    TypeVar,
    Union,
)

import orjson
from loguru import logger
from pydantic import BaseModel, PrivateAttr
from typing_extensions import TypeAlias, override

from sphinx_jsx.utils.pydantic import update_forward_refs

JSON: TypeAlias = Union[Dict, List, str, int, float, bool, None]

TElement = TypeVar("TElement", bound="JSXElement")
UElement = TypeVar("UElement", bound="JSXElement")


def json_with_jsx(obj: Any, *, default: Optional[Callable] = None) -> str:
    def _default(x: Any):
        if isinstance(x, JSFragment):
            return orjson.Fragment(x.render())
        if default:
            return default(x)
        raise TypeError(f"cannot serialize type: {type(x)}: {repr(x)}")

    return orjson.dumps(obj, default=_default).decode()


class JSFragment(
    BaseModel,
    ABC,
    validate_assignment=True,
    json_dumps=json_with_jsx,
):
    @abstractmethod
    def render(self) -> str:
        raise NotImplementedError


class UnsafeInline(JSFragment):
    kind: Literal["unsafe"] = "unsafe"
    value: str

    @override
    def render(self) -> str:
        return self.value


class JSONExpression(JSFragment):
    kind: Literal["json"] = "json"
    value: Any

    @override
    def render(self) -> str:
        return json_with_jsx(self.value)

    def __repr__(self) -> str:
        return repr(self.value)


class JSXAttribute(JSFragment):
    kind: Literal["jsx_attribute"] = "jsx_attribute"

    name: str
    value: JSFragment

    @override
    def render(self) -> str:
        return f"{self.name}={{{self.value.render()}}}"


class JSXElement(JSFragment):
    kind: Literal["jsx_element"] = "jsx_element"

    _tag_name: ClassVar[str]
    _tag_namespace: ClassVar[Optional[Type[JSXElement]]] = None

    ids: List[str] = []
    classnames: List[str] = []
    children: List[JSFragment] = []

    _style: Optional[str] = PrivateAttr(None)
    _attrs: Dict[str, Any] = PrivateAttr({})

    @classmethod
    def __init_subclass__(cls, *, tag_name: Optional[str] = None, **kwargs):
        super().__init_subclass__(**kwargs)
        cls._tag_name = tag_name or cls.__name__
        for value in vars(cls).values():
            try:
                if JSXElement in value.mro():
                    value._tag_namespace = cls
            except (AttributeError, TypeError):
                continue

    @classmethod
    def tag_name(cls):
        if not cls._tag_namespace:
            return cls._tag_name
        return f"{cls._tag_namespace.tag_name()}.{cls._tag_name}"

    @classmethod
    def get_tag_by_name(cls, name: str) -> Optional[Type[JSXElement]]:
        for value in vars(cls).values():
            try:
                if value._tag_name == name:
                    return value
            except AttributeError:
                continue

    def props(self) -> Iterator[JSXAttribute]:
        # TODO: jinja
        if self.classnames:
            expr = JSONExpression(value=" ".join(filter(None, self.classnames)))
            yield JSXAttribute(name="className", value=expr)

        if self.ids:
            primary_id, *secondary_ids = self.ids
            expr = JSONExpression(value=primary_id)
            yield JSXAttribute(name="id", value=expr)

            if secondary_ids:
                expr = JSONExpression(value=secondary_ids)
                yield JSXAttribute(name="extraIds", value=expr)

        for key, value in chain(getattr(self, "_attrs", {}).items(), self):
            if (
                key in {"kind", "ids", "classnames", "children", "style"}
                or value is None
            ):
                continue

            if isinstance(value, JSFragment):
                yield JSXAttribute(name=key, value=value)

            else:
                yield JSXAttribute(name=key, value=JSONExpression(value=value))

    @override
    def render(self) -> str:
        # TODO: jinja
        props = " ".join((p.render() for p in self.props()))
        if props:
            start_tag = f"<{self.tag_name()} {props}>"
        else:
            start_tag = f"<{self.tag_name()}>"
        children = (
            f"{{{c.render()}}}" if not isinstance(c, JSXElement) else c.render()
            for c in self.children
        )
        end_tag = f"</{self.tag_name()}>"
        return "".join((start_tag, *children, end_tag))

    def __repr__(self):
        return self._tag_name


class PseudoElement(JSXElement):
    kind: Literal["jsx_pseudo_element"] = "jsx_pseudo_element"

    @override
    def render(self) -> str:
        logger.warning(f"rendering pseudo element: {repr(self)}")
        return ""


class ESMExport(JSFragment):
    kind: Literal["esm_export"] = "esm_export"

    name: str
    value: Union[JSONExpression, UnsafeInline]

    @override
    def render(self) -> str:
        return f"export const {self.name} = {self.value.render()};"


class ESMImport(JSFragment):
    kind: Literal["esm_import"] = "esm_import"

    name: str
    source: str

    @override
    def render(self) -> str:
        source = orjson.dumps(self.source).decode()
        return f"import {{ {self.name} }} from {source};"


class JSObject(JSFragment):
    @override
    def render(self) -> str:
        return self.json(models_as_dict=False, exclude_none=True)


class React(PseudoElement):
    class Fragment(JSXElement):
        @override
        def render(self) -> str:
            if not self.children:
                return ""
            if len(self.children) == 1:
                return self.children[0].render()
            return f'<>{"".join([c.render() for c in self.children])}</>'


class InvalidNestingError(ValueError):
    def __init__(self, parent: JSXElement, *children: JSFragment) -> None:
        self.parent = parent
        self.children = children

    def __str__(self) -> str:
        children = ", ".join(repr(c) for c in self.children)
        return f"invalid nesting in {self.parent.tag_name()}: {children}"


class EmptySelectionError(ValueError):
    def __init__(self, context: JSXElement, tag_type: Type[JSXElement]) -> None:
        self.context = context
        self.tag_type = tag_type

    def __str__(self) -> str:
        return (
            f"Expected <{self.tag_type.tag_name()}> in {repr(self.context)},"
            " but none found."
        )


JSON_NULL = JSONExpression(value=None)

update_forward_refs(globals())
