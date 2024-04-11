from __future__ import annotations

from typing import Iterator, Optional, Set

from docutils import nodes
from loguru import logger
from sphinx.application import Sphinx
from sphinx.builders import Builder
from sphinx.builders.html import BuildInfo
from sphinx.environment import BuildEnvironment

from ._lib import Bundler
from .options import SphinxConfig, SphinxOptions
from .translator import SphinxJSXTranslator


class SphinxJSXBuilder(Builder):
    name = "jsx"
    format = "jsx"

    default_translator_class = SphinxJSXTranslator

    versioning_method = "none"
    versioning_compare = False

    use_message_catalog = True

    supported_image_types = []
    supported_remote_images = True
    supported_data_uri_images = True

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)
        self.build_info: BuildInfo
        self.bundler = Bundler(
            SphinxOptions(
                conf=SphinxConfig(
                    extensions=[*app.extensions.keys()],
                    myst_enable_extensions=getattr(
                        app.config, "myst_enable_extensions", None
                    ),
                ),
                srcdir=str(self.srcdir),
                outdir=str(self.outdir),
            )
        )

    def get_target_uri(self, docname: str, typ: Optional[str] = None) -> str:
        return docname

    def init(self) -> None:
        self.build_info = self.create_build_info()
        self.source_map = self.bundler.sourcemap()

    def create_build_info(self) -> BuildInfo:
        return BuildInfo(self.config, self.tags, ["jsx"])

    def get_outdated_docs(self) -> Iterator[str]:
        yield from self.env.found_docs

    def prepare_writing(self, docnames: Set[str]) -> None:
        return

    @logger.catch(reraise=True)
    def write_doc(self, docname: str, doctree: nodes.document) -> None:
        translator = self.create_translator(doctree, self, docname)

        if not isinstance(translator, SphinxJSXTranslator):
            raise TypeError("translator must be a subclass of SphinxJSXTranslator")

        doctree.walkabout(translator)

    def finish(self):
        self.bundler.build(self.source_map)
