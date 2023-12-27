from __future__ import annotations

import mimetypes
from typing import Dict, List, Optional, Tuple, Union
from urllib.parse import urlsplit

from docutils import nodes
from more_itertools import first

from sphinx_jsx.syntax.jsx.models import JSON_NULL, JSFragment, JSObject, JSXElement
from sphinx_jsx.utils.pydantic import update_forward_refs

from .base import BaseJSXTranslator
from .pseudo import Pseudo


class Figure(JSXElement):
    width: Optional[str] = None
    align: Optional[str] = None

    caption: JSFragment = JSON_NULL
    legend: JSFragment = JSON_NULL

    class Image(JSXElement):
        class Source(JSObject):
            src: str
            type: Optional[str] = None

            width: Optional[int] = None
            height: Optional[int] = None
            density: Optional[float] = None

        src: str
        srcset: List[Figure.Image.Source] = []
        alt: Optional[str] = None


class FigureMarkupTranslator(BaseJSXTranslator):
    def visit_image(self, node: nodes.image):
        candidates: Dict[str, str] = node["candidates"]

        srcset = [
            Figure.Image.Source(src=src, type=guess_mimetype(src))
            for src in candidates.values()
        ]

        if not srcset:
            raise nodes.SkipNode

        alt = node.get("alt", None)
        src = select_src(srcset).src
        self.append_child(node, Figure.Image(srcset=srcset, src=src, alt=alt))

        for src in srcset:
            self.add_reference(src.src)

    def visit_figure(self, node: nodes.figure):
        figure = self.enter_nesting(node, Figure())
        figure.width = node.get("width", None)
        figure.align = node.get("align", None)

    def visit_legend(self, node: nodes.legend):
        self.enter_nesting(node, Pseudo._Legend())

    def visit_caption(self, node: nodes.caption):
        self.enter_nesting(node, Pseudo._Caption())

    def depart_figure(self, node: nodes.figure):
        elem = self.leave_nesting(node, Figure)
        elem.caption = elem.remove_fragment(Pseudo._Caption)
        elem.legend = elem.remove_fragment(Pseudo._Legend)


def guess_mimetype(src: str) -> str:
    try:
        path = urlsplit(src).path
    except ValueError:
        path = src
    return mimetypes.guess_type(path)[0] or "application/octet-stream"


def select_src(srcset: List[Figure.Image.Source]) -> Figure.Image.Source:
    def keyfunc(src: Figure.Image.Source) -> Tuple[Union[int, float], ...]:
        type_preference = [
            "image/jpeg",
            "image/gif",
            "image/png",
            "image/tiff",
            "image/webp",
            "image/svg+xml",
            "video/mp4",
            "video/webm",
        ]
        try:
            preference = type_preference.index(src.type or "application/octet-stream")
        except ValueError:
            preference = 0
        return (preference, src.width or 0, src.height or 0, src.density or 1)

    return first(sorted(srcset, key=keyfunc, reverse=True))


update_forward_refs(globals())
