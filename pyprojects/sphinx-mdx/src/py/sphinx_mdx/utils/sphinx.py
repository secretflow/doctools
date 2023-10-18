from typing import Callable, Literal, Optional, Type

from docutils.nodes import Element
from sphinx.application import Sphinx

from sphinx_mdx.translator import MDXTranslator


class SkipHandler(Exception):
    pass


def override_handlers(
    app: Sphinx,
    elem: Type[Element],
    on_visit: Callable,
    on_depart: Optional[Callable] = None,
    mode: Literal["greedy", "after"] = "greedy",
):
    handlers = app.registry.translation_handlers.get("mdx", {})
    visit, depart = handlers.get(elem.__name__, (None, None))

    if not visit:
        visit = getattr(
            MDXTranslator,
            "visit_" + elem.__name__,
            MDXTranslator.unknown_visit,
        )

    if not depart:
        depart = getattr(
            MDXTranslator,
            "depart_" + elem.__name__,
            MDXTranslator.unknown_departure,
        )

    if mode == "greedy":

        def override_visit(self, node):
            try:
                on_visit(self, node)
            except SkipHandler:
                visit(self, node)

        def override_depart(self, node):
            try:
                if not on_depart:
                    raise SkipHandler
                on_depart(self, node)
            except SkipHandler:
                depart(self, node)

    else:

        def override_visit(self, node):
            visit(self, node)
            on_visit(self, node)

        def override_depart(self, node):
            depart(self, node)
            if on_depart:
                on_depart(self, node)

    app.registry.add_translation_handlers(elem, mdx=(override_visit, override_depart))
