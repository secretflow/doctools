from __future__ import annotations

import os
from datetime import datetime
from os import path
from pathlib import Path
from typing import Iterator, Literal, Optional, Set, cast

from docutils import nodes
from pydantic import BaseModel
from ruamel.yaml import YAML
from sphinx import addnodes
from sphinx.application import Sphinx
from sphinx.builders import Builder
from sphinx.builders.html import INVENTORY_FILENAME, BuildInfo
from sphinx.environment import BuildEnvironment
from sphinx.locale import __
from sphinx.util.display import progress_message, status_iterator
from sphinx.util.inventory import InventoryFile

from .options import parse_options
from .scaffolding import Scaffolding
from .sidebar import Sidebar, generate_sidebar
from .translator import MDXTranslator
from .utils.logging import get_logger

yaml = YAML(typ="safe", pure=True)
yaml.indent(mapping=2, sequence=4, offset=2)

logger = get_logger(__name__)


class Manifest(BaseModel):
    version: Literal["1"] = "1"
    sidebar: Sidebar


class MDXBuilder(Builder):
    name = "mdx"
    format = "mdx"

    default_translator_class = MDXTranslator

    versioning_method = "none"
    versioning_compare = False

    use_message_catalog = True

    supported_image_types = []
    supported_remote_images = True
    supported_data_uri_images = True

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)
        self.build_info: BuildInfo

        self.build_env = parse_options(app, self)
        self.scaffolding: Scaffolding

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
        return self.scaffolding.get_target_uri(docname)

    def init(self) -> None:
        self.build_info = self.create_build_info()
        self.scaffolding = Scaffolding.create(self.build_env)

    def create_build_info(self) -> BuildInfo:
        return BuildInfo(self.config, self.tags, ["mdx"])

    def get_outdated_docs(self) -> Iterator[str]:
        # mostly from sphinx

        if os.getenv("SPHINX_MDX_DEVELOPMENT"):
            yield from self.env.found_docs

        try:
            with open(self.build_info_path, encoding="utf-8") as fp:
                buildinfo = BuildInfo.load(fp)
            if self.build_info != buildinfo:
                logger.debug("[build target] did not match: build_info ")
                yield from self.env.found_docs
                return
        except ValueError as exc:
            logger.warning(__("Failed to read build info file: %r"), exc)
        except OSError:
            # ignore errors on reading
            pass

        for docname in self.env.found_docs:
            if docname not in self.env.all_docs:
                logger.debug("[build target] not in env: %r", docname)
                yield docname
                continue
            output_path = self.scaffolding.get_output_path(docname)
            try:
                output_mtime = path.getmtime(output_path)
            except Exception:
                output_mtime = 0
            try:
                source_mtime = path.getmtime(self.env.doc2path(docname))
                if source_mtime > output_mtime:
                    logger.debug(
                        "[build target] target %r (%s), docname %r (%s)",
                        output_mtime,
                        datetime.utcfromtimestamp(output_mtime),
                        docname,
                        datetime.utcfromtimestamp(source_mtime),
                    )
                    yield docname
            except OSError:
                # source doesn't exist anymore
                pass

    def prepare_writing(self, docnames: Set[str]) -> None:
        # rebuild staticfiles from doctrees

        progress = status_iterator(
            self.env.all_docs,
            "resolving assets ... ",
            "darkgreen",
            len(self.env.all_docs),
            self.app.verbosity,
        )

        for docname in progress:
            doctree = self.env.get_doctree(docname)
            for node in doctree.findall(nodes.image):
                self.scaffolding.add_image(docname, node)
            for node in doctree.findall(addnodes.download_reference):
                self.scaffolding.add_downloadable_file(docname, node)

    def write_doc(self, docname: str, doctree: nodes.document) -> None:
        translator = cast(MDXTranslator, self.create_translator())
        doctree.walkabout(translator)
        self.scaffolding.add_doc(docname, translator.root)

    def copy_assets(self) -> None:
        self.scaffolding.copy_assets()

    @progress_message(__("dumping object inventory"))
    def dump_inventory(self) -> None:
        InventoryFile.dump(str(self.output_root / INVENTORY_FILENAME), self.env, self)

    def write_manifest(self):
        manifest_path = self.output_root / "manifest.yml"

        # Generate sidebar
        sidebar_docname = self.config.root_doc
        doctree = self.env.get_doctree(sidebar_docname)
        sidebar = generate_sidebar(doctree, self.scaffolding, self.env)

        manifest = Manifest(sidebar=sidebar)

        with open(manifest_path, "w+") as f:
            yaml.dump(manifest.dict(exclude_none=True), f)

    def write_buildinfo(self) -> None:
        try:
            with open(self.build_info_path, "w", encoding="utf-8") as fp:
                self.build_info.dump(fp)
        except OSError as exc:
            logger.warning(__("Failed to write build info file: %r"), exc)

    def finish(self):
        self.finish_tasks.add_task(self.copy_assets)
        self.finish_tasks.add_task(self.dump_inventory)
        self.finish_tasks.add_task(self.write_buildinfo)
        self.finish_tasks.add_task(self.write_manifest)
