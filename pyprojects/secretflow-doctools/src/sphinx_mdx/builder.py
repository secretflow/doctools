from __future__ import annotations

import os
import re
import shutil
import subprocess
from contextlib import suppress
from datetime import datetime
from os import path
from pathlib import Path
from typing import Iterator, Literal, Optional, Set, Tuple, Union
from urllib.parse import urlunsplit

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

from .mdserver.client import MarkdownClient
from .options import parse_options
from .pathfinding import Pathfinder, StaticFiles
from .sidebar import Sidebar, generate_sidebar
from .translator import MDXTranslator
from .utils.logging import get_logger

yaml = YAML(typ="safe", pure=True)
yaml.indent(mapping=2, sequence=4, offset=2)


class Manifest(BaseModel):
    version: Literal["1"] = "1"
    sidebar: Sidebar


def ensure_parent(path: Union[Path, str]) -> None:
    """Ensure that the parent of a path exists."""
    os.makedirs(Path(path).parent, exist_ok=True)


class MDXBuilder(Builder):
    name = "mdx"
    format = "mdx"

    default_translator_class = MDXTranslator

    versioning_method = "none"
    versioning_compare = False

    use_message_catalog = True

    supported_image_types = []
    """Indicate images that are supported by the builder's output format, so
    that Sphinx will not attempt to convert them.

    Since we are generating JavaScript source code, resources are handled by
    the downstream bundler and loaders, we use an empty list to indicate we
    support all images.
    """

    supported_remote_images = True
    """If set to False, Sphinx will attempt to download remote images. We are
    building web apps so this is necessarily True."""

    supported_data_uri_images = True
    """If set to False, Sphinx will attempt to convert data URIs to image files.
    Browsers support data URIs, so we set this to True.
    """

    file_extension = ".mdx"

    logger = get_logger(__name__)

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)
        self.options = parse_options(app.config)

        self.build_info: BuildInfo

        self.mdclient = MarkdownClient(self.options.mdx_mdserver_origin)
        self.pathfinder: Pathfinder
        self.staticfiles: StaticFiles

        self.git_host: Optional[str] = None
        self.git_owner: Optional[str] = None
        self.git_repo: Optional[str] = None
        self.git_root: Optional[Path] = None
        self.git_revision_commit: Optional[str] = None
        self.git_revision_time: Optional[str] = None

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
        return self.pathfinder.get_target_uri(docname)

    def init(self) -> None:
        self.build_info = self.create_build_info()
        self.pathfinder = Pathfinder(self, self.options)
        self.staticfiles = StaticFiles(self, self.options)
        self.mdclient.start()

        with suppress(subprocess.CalledProcessError):
            git_origin = (
                subprocess.run(
                    ["git", "config", "--get", "remote.origin.url"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )
            match = re.match(
                r"git@(?:.*)github\.com:(?P<owner>[^/]+)/(?P<repo>[^/]+)",
                git_origin,
            ) or re.match(
                r"https://github\.com/(?P<owner>[^/]+)/(?P<repo>[^/]+)",
                git_origin,
            )
            if match:
                owner = match["owner"]
                repo = match["repo"]
                if repo.endswith(".git"):
                    repo = repo[:-4]
                self.git_host = "github.com"
                self.git_owner = owner
                self.git_repo = repo
        with suppress(subprocess.CalledProcessError):
            self.git_revision_commit = (
                subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )
        with suppress(subprocess.CalledProcessError):
            self.git_root = Path(
                subprocess.run(
                    ["git", "rev-parse", "--show-toplevel"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )
        with suppress(subprocess.CalledProcessError):
            self.git_revision_time = (
                subprocess.run(
                    ["git", "log", "-1", "--format=%cI"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )

    def get_source_tree_path(self, docname: str) -> Optional[str]:
        if not self.git_root:
            return None
        path = self.env.doc2path(docname, base=False)
        path = self.source_root.joinpath(path)
        try:
            return str(path.relative_to(self.git_root))
        except ValueError:
            return None

    def get_origin_url(self, docname: str) -> Optional[str]:
        if not self.git_revision_commit or self.git_host != "github.com":
            return None
        if not (path := self.get_source_tree_path(docname)):
            return None
        path = (
            f"/{self.git_owner}/{self.git_repo}/blob/{self.git_revision_commit}/{path}"
        )
        return urlunsplit(("https", self.git_host, path, "", ""))

    def get_download_url(self, docname: str) -> Optional[str]:
        if not self.git_revision_commit or self.git_host != "github.com":
            return None
        if not (path := self.get_source_tree_path(docname)):
            return None
        path = (
            f"/{self.git_owner}/{self.git_repo}/raw/{self.git_revision_commit}/{path}"
        )
        return urlunsplit(("https", self.git_host, path, "", ""))

    def get_last_modified(self, docname: str) -> Tuple[Optional[str], Optional[str]]:
        if not self.git_revision_commit or self.git_host != "github.com":
            return (None, None)
        if not (path := self.get_source_tree_path(docname)):
            return (None, None)
        try:
            info = (
                subprocess.run(
                    ["git", "log", "-1", r"--format=%cI%%%H", "--", f":/{path}"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )
            date, sha = info.split("%")
            return (date, sha)
        except (subprocess.CalledProcessError, ValueError):
            return (None, None)

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
                self.logger.debug("[build target] did not match: build_info ")
                yield from self.env.found_docs
                return
        except ValueError as exc:
            self.logger.warning(__("Failed to read build info file: %r"), exc)
        except OSError:
            # ignore errors on reading
            pass

        for docname in self.env.found_docs:
            if docname not in self.env.all_docs:
                self.logger.debug("[build target] not in env: %r", docname)
                yield docname
                continue
            output_path = self.pathfinder.get_output_path(docname)
            try:
                output_mtime = path.getmtime(output_path)
            except Exception:
                output_mtime = 0
            try:
                source_mtime = path.getmtime(self.env.doc2path(docname))
                if source_mtime > output_mtime:
                    self.logger.debug(
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
                self.staticfiles.add_image(docname, node)
            for node in doctree.findall(addnodes.download_reference):
                self.staticfiles.add_downloadable_file(docname, node)

    def write_doc(self, docname: str, doctree: nodes.document) -> None:
        output_path = self.pathfinder.get_output_path(docname)
        ensure_parent(output_path)

        translator = MDXTranslator(
            doctree,
            self,
            self.options,
            self.mdclient,
            self.pathfinder,
            self.staticfiles,
        )
        doctree.walkabout(translator)

        origin_url = self.get_origin_url(docname)
        if origin_url:
            last_modified_date, last_modified_commit = self.get_last_modified(docname)
            translator.metadata.update(
                {
                    "git_origin_url": origin_url,
                    "git_download_url": self.get_download_url(docname),
                    "git_owner": self.git_owner,
                    "git_repo": self.git_repo,
                    "git_revision_commit": self.git_revision_commit,
                    "git_revision_time": self.git_revision_time,
                    "git_last_modified_commit": last_modified_commit,
                    "git_last_modified_time": last_modified_date,
                }
            )

        output_path.write_text(translator.export())

    def copy_assets(self) -> None:
        for src, dst in self.staticfiles.files.items():
            ensure_parent(dst)
            shutil.copy(src, dst)

    @progress_message(__("dumping object inventory"))
    def dump_inventory(self) -> None:
        InventoryFile.dump(str(self.output_root / INVENTORY_FILENAME), self.env, self)

    def write_manifest(self):
        manifest_path = self.output_root / "manifest.yml"

        # Generate sidebar
        sidebar_docname = self.config.root_doc
        doctree = self.env.get_doctree(sidebar_docname)
        sidebar = generate_sidebar(doctree, self.pathfinder, self.env)

        manifest = Manifest(sidebar=sidebar)

        with open(manifest_path, "w+") as f:
            yaml.dump(manifest.dict(exclude_none=True), f)

    def write_buildinfo(self) -> None:
        try:
            with open(self.build_info_path, "w", encoding="utf-8") as fp:
                self.build_info.dump(fp)
        except OSError as exc:
            self.logger.warning(__("Failed to write build info file: %r"), exc)

    def finish(self):
        self.finish_tasks.add_task(self.copy_assets)
        self.finish_tasks.add_task(self.dump_inventory)
        self.finish_tasks.add_task(self.write_buildinfo)
        self.finish_tasks.add_task(self.write_manifest)

    def cleanup(self) -> None:
        self.mdclient.stop()
