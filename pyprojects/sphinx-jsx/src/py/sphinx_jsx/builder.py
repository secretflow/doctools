from __future__ import annotations

from pathlib import Path
from typing import Iterator, Optional, Set

from docutils import nodes
from loguru import logger
from sphinx.application import Sphinx
from sphinx.builders import Builder
from sphinx.builders.html import INVENTORY_FILENAME, BuildInfo
from sphinx.environment import BuildEnvironment
from sphinx.locale import __
from sphinx.util.display import progress_message
from sphinx.util.inventory import InventoryFile

from .options import parse_options


class SphinxJSXBuilder(Builder):
    name = "jsx"
    format = "jsx"

    # default_translator_class = SphinxJSXTranslator

    versioning_method = "none"
    versioning_compare = False

    use_message_catalog = True

    supported_image_types = []
    supported_remote_images = True
    supported_data_uri_images = True

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)
        self.build_info: BuildInfo
        self.build_options = parse_options(app, self)

    @property
    def source_root(self) -> Path:
        return Path(self.srcdir)

    @property
    def output_root(self) -> Path:
        return Path(self.outdir)

    @property
    def build_info_path(self):
        return self.output_root / ".buildinfo"

    def get_target_uri(self, docname: str, typ: Optional[str] = None) -> str:
        return docname

    def init(self) -> None:
        self.build_info = self.create_build_info()

    def create_build_info(self) -> BuildInfo:
        return BuildInfo(self.config, self.tags, ["jsx"])

    def get_outdated_docs(self) -> Iterator[str]:
        # mostly from sphinx
        yield from self.env.found_docs

    def prepare_writing(self, docnames: Set[str]) -> None:
        return

    @logger.catch(reraise=True)
    def write_doc(self, docname: str, doctree: nodes.document) -> None:
        raise NotImplementedError
        # translator = cast(SphinxJSXTranslator, self.create_translator(doctree, self))
        # doctree.walkabout(translator)
        # translator.render()

    @progress_message(__("dumping object inventory"))
    def dump_inventory(self) -> None:
        InventoryFile.dump(str(self.output_root / INVENTORY_FILENAME), self.env, self)

    def write_buildinfo(self) -> None:
        try:
            with open(self.build_info_path, "w", encoding="utf-8") as fp:
                self.build_info.dump(fp)
        except OSError as exc:
            logger.warning(__("Failed to write build info file: %r"), exc)

    def finish(self):
        self.finish_tasks.add_task(self.dump_inventory)
        self.finish_tasks.add_task(self.write_buildinfo)
