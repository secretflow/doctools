from __future__ import annotations

import math
from typing import Any, Dict, List, Literal, Optional, Union

from pydantic import BaseModel, Field
from typing_extensions import Annotated

from sphinx_jsx.utils.pydantic import ORJSONConfig, update_forward_refs

u32 = Annotated[int, Field(ge=0, le=4294967295)]
ident = Annotated[str, Field(regex=r"^[a-zA-Z_$][a-zA-Z0-9_$]*$")]


class ASTItem(BaseModel):
    class Config(ORJSONConfig):
        pass


class Span(ASTItem):
    """https://rustdoc.swc.rs/swc_common/source_map/struct.Span.html"""

    start: u32
    end: u32
    ctxt: u32


class ExprBase(ASTItem):
    ...
    span: Span = Span(start=0, end=0, ctxt=0)


class Str(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.Str.html"""

    type: Literal["StringLiteral"] = "StringLiteral"

    value: str
    raw: Optional[str] = None


class Bool(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.Bool.html"""

    type: Literal["BooleanLiteral"] = "BooleanLiteral"

    value: bool


class Null(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.Null.html"""

    type: Literal["NullLiteral"] = "NullLiteral"


class Number(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.Number.html"""

    type: Literal["NumericLiteral"] = "NumericLiteral"

    value: float
    raw: Optional[str] = None


class ArrayLit(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.ArrayLit.html"""

    type: Literal["ArrayExpression"] = "ArrayExpression"

    elements: List[ExprOrSpread]


class ExprOrSpread(ASTItem):
    """https://rustdoc.swc.rs/swc_core/ecma/ast/struct.ExprOrSpread.html"""

    spread: Optional[Span] = None
    expression: Expr


class ObjectLit(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.ObjectLit.html"""

    type: Literal["ObjectExpression"] = "ObjectExpression"

    properties: List[KeyValueProp]


class KeyValueProp(ASTItem):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.KeyValueProp.html"""

    type: Literal["KeyValueProperty"] = "KeyValueProperty"

    key: Union[Str, Ident]
    value: Expr


class CallExpr(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.CallExpr.html"""

    type: Literal["CallExpression"] = "CallExpression"

    callee: Ident
    arguments: List[ExprOrSpread]

    typeArguments: Optional[Any] = None


class Ident(ExprBase):
    """https://rustdoc.swc.rs/swc_ecma_ast/struct.Ident.html"""

    type: Literal["Identifier"] = "Identifier"

    value: ident

    optional: Optional[bool] = False


Expr = Annotated[
    Union[Str, Bool, Null, Number, ArrayLit, ObjectLit, CallExpr, Ident],
    Field(discriminator="type"),
]
"""https://rustdoc.swc.rs/swc_core/ecma/utils/swc_ecma_ast/enum.Expr.html"""

JSONPrimitive = Union[str, bool, int, float, None]
JSONType = Union[JSONPrimitive, List["ECMAType"], Dict[str, "ECMAType"]]
ECMAType = Union[JSONType, Expr]


def ecma(value: ECMAType) -> Expr:
    # I want PEP 635. Or std::convert::Into.
    if isinstance(value, ExprBase):
        return value
    if isinstance(value, list):
        return ArrayLit(
            elements=[ExprOrSpread(expression=ecma(value)) for value in value]
        )
    if isinstance(value, dict):
        return ObjectLit(
            properties=[
                KeyValueProp(key=Str(value=k), value=ecma(v)) for k, v in value.items()
            ]
        )
    if value is None:
        return Null()
    if isinstance(value, str):
        return Str(value=value)
    if isinstance(value, bool):
        return Bool(value=value)
    if isinstance(value, int):
        return Number(value=value)
    if isinstance(value, float):
        if math.isnan(value):
            return Ident(value="NaN")
        if math.isinf(value):
            return Ident(value="Infinity")
        return Number(value=value)
    raise TypeError(f"Unsupported literal type: {type(value)}")


def ecma_call(callee: ident, *args: ECMAType) -> CallExpr:
    return CallExpr(
        callee=Ident(value=callee),
        arguments=[ExprOrSpread(expression=ecma(expr)) for expr in args],
    )


update_forward_refs(locals())
