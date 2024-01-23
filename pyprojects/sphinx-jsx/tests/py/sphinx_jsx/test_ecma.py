import cProfile
import pstats
import timeit
from textwrap import dedent

from sphinx_jsx._lib import testing
from sphinx_jsx.ecma import Ident, ecma, ecma_call


def test_ecma_literal():
    assert (
        testing.ast_string_to_ecma(
            ecma(
                {
                    "string": "the lazy dog",
                    "number": 42,
                    "bool": True,
                    "null": None,
                    "float": 2**46 + 0.3,
                    "inf": float("inf"),
                    "nan": float("nan"),
                    "array": [1, "2", {"3": 4}],
                }
            ).json()
        )
        == dedent(
            """
            {
                "string": "the lazy dog",
                "number": 42,
                "bool": true,
                "null": null,
                "float": 70368744177664.3,
                "inf": Infinity,
                "nan": NaN,
                "array": [
                    1,
                    "2",
                    {
                        "3": 4
                    }
                ]
            }
            """
        ).strip()
    )


def test_ecma_call():
    call = ecma_call(
        "jsx",
        "a",
        {
            "href": ecma_call("_url", "external", None, "https://swc.rs/"),
            "children": ecma_call(
                "jsx", Ident(value="Fragment"), {"children": "Speedy Web Compiler"}
            ),
        },
    )
    assert (
        testing.ast_string_to_ecma(call.json())
        == dedent(
            """\
            jsx("a", {
                "href": _url("external", null, "https://swc.rs/"),
                "children": jsx(Fragment, {
                    "children": "Speedy Web Compiler"
                })
            })
            """
        ).strip()
    )


def bench():
    call = ecma_call(
        "jsx",
        "a",
        {
            "href": ecma_call("_url", "external", None, "https://swc.rs/"),
            "children": ecma_call(
                "jsx", Ident(value="Fragment"), {"children": "Speedy Web Compiler"}
            ),
        },
    ).json()

    def to_bench():
        testing.ast_string_to_ecma(call)

    with cProfile.Profile() as pr:
        print(timeit.Timer(to_bench).autorange())

    stats = pstats.Stats(pr)
    stats.dump_stats("bench.prof")
