from __future__ import annotations

import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Annotated, Iterator, Literal, Optional, Union

from docutils import nodes
from loguru import logger
from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel
from ruamel.yaml import YAML
from sphinx import addnodes
from sphinx.application import Sphinx
from sphinx.builders import Builder
from sphinx.builders.html import (
    INVENTORY_FILENAME,
    BuildInfo,  # pyright: ignore [reportPrivateImportUsage]
)
from sphinx.environment import BuildEnvironment
from sphinx.locale import __
from sphinx.util.display import progress_message, status_iterator
from sphinx.util.inventory import InventoryFile

from secretflow_doctools.i18n.builder import SwaggerGettextBuilder
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.markdown.ffi import MarkdownClient
from secretflow_doctools.options import GettextConfig, MdxConfig, parse_config
from secretflow_doctools.pathfinding import Pathfinder, StaticFiles
from secretflow_doctools.sidebar import Sidebar, generate_sidebar
from secretflow_doctools.translator import MdxTranslator
from secretflow_doctools.utils.logging import configure_logging
from secretflow_doctools.vcs import RemoteVCS, git_origin, guess_remote_vcs

yaml = YAML(typ="safe", pure=True)
yaml.indent(mapping=2, sequence=4, offset=2)


class PageMetadata(BaseModel):
    git_origin_url: str
    git_download_url: Optional[str] = None
    git_owner: Optional[str] = None
    git_repo: Optional[str] = None
    git_revision_commit: Optional[str] = None
    git_revision_time: Optional[str] = None
    git_last_modified_commit: Optional[str] = None
    git_last_modified_time: Optional[str] = None
    page_dependencies: list[PageDependency] = []


class PageDependency(BaseModel):
    type: Literal["content", "gettext", "attachment"]
    path: str
    time: Optional[
        Annotated[Union[RevisionTime, ModifiedTime], Field(discriminator="type")]
    ] = None


class RevisionTime(BaseModel):
    type: Literal["revision"] = "revision"
    time: str
    commit: str


class ModifiedTime(BaseModel):
    type: Literal["modified"] = "modified"
    time: str


class Manifest(BaseModel):
    version: Literal["2"] = "2"
    sidebar: Sidebar
    project_name: str

    model_config = ConfigDict(alias_generator=to_camel, populate_by_name=True)


def ensure_parent(path: Union[Path, str]) -> None:
    """Ensure that the parent of a path exists."""
    os.makedirs(Path(path).parent, exist_ok=True)


class MdxBuilder(Builder):
    name = "mdx"
    format = "mdx"

    default_translator_class = MdxTranslator

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

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)

        configure_logging()

        self.options = parse_config(app.config, MdxConfig)

        self.build_info: BuildInfo

        self.ffi = MarkdownClient()
        self.pathfinder: Pathfinder
        self.static_files: StaticFiles
        self.swagger_i18n: SwaggerGettextBuilder

        self.git_remote: Optional[RemoteVCS] = None
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
        self.static_files = StaticFiles(self, self.options)
        self.swagger_i18n = SwaggerGettextBuilder(self.app, self.env)
        self.ffi.start()

        if origin := git_origin():
            self.git_remote = guess_remote_vcs(origin)

        with logger.catch(level="WARNING", message=_("failed to get git commit")):
            self.git_revision_commit = (
                subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )

        with logger.catch(level="WARNING", message=_("failed to get git commit time")):
            self.git_revision_time = (
                subprocess.run(
                    ["git", "log", "-1", "--format=%cI"],
                    capture_output=True,
                    check=True,
                )
                .stdout.decode("utf-8")
                .strip()
            )

        with logger.catch(level="WARNING", message=_("failed to get git root dir")):
            self.git_root = Path(
                subprocess.run(
                    ["git", "rev-parse", "--show-toplevel"],
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
        if not self.git_revision_commit or not self.git_remote:
            return None
        if not (path := self.get_source_tree_path(docname)):
            return None
        return self.git_remote.permalink(self.git_revision_commit, path, kind="tree")

    def create_build_info(self) -> BuildInfo:
        return BuildInfo(self.config, self.tags, {"env"})

    def get_outdated_docs(self) -> Iterator[str]:
        # mostly from sphinx

        try:
            buildinfo = BuildInfo.load(self.build_info_path)
            if self.build_info != buildinfo:
                logger.debug("[build target] did not match: build_info ")
                yield from self.env.found_docs
                return
        except ValueError as exc:
            logger.warning(__("Failed to read build info file: %r") % (exc,))
        except OSError:
            # ignore errors on reading
            pass

        for docname in self.env.found_docs:
            if docname not in self.env.all_docs:
                logger.debug("[build target] not in env: %r", docname)
                yield docname
                continue

            if is_outdated(
                self.pathfinder.get_output_path(docname),
                self.env.doc2path(docname),
                self.swagger_i18n.get_messages_path(docname),
            ):
                yield docname

    def write_documents(self, docnames):
        # rebuild staticfiles from doctrees

        progress = status_iterator(
            self.env.all_docs,
            "resolving media files ... ",
            "darkgreen",
            len(self.env.all_docs),
            self.app.verbosity,
        )

        for docname in progress:
            doctree = self.env.get_doctree(docname)
            for node in doctree.findall(nodes.image):
                self.static_files.add_image(docname, node)
            for node in doctree.findall(addnodes.download_reference):
                self.static_files.add_downloadable_file(docname, node)

        super().write_documents(docnames)

    def write_doc(self, docname: str, doctree: nodes.document) -> None:
        output_path = self.pathfinder.get_output_path(docname)
        ensure_parent(output_path)

        translator = MdxTranslator(
            doctree,
            self,
            self.options,
            self.ffi,
            self.pathfinder,
            self.static_files,
            self.swagger_i18n,
        )

        doctree.walkabout(translator)

        if metadata := self.resolve_metadata(docname):
            translator.metadata.update(metadata.model_dump())

        with open(output_path, "wb+") as f:
            f.write(translator.export().encode("utf-8"))

    def resolve_metadata(self, docname: str) -> Optional[PageMetadata]:
        if not self.git_revision_commit or not self.git_remote or not self.git_root:
            return None
        if not (content_path := self.get_source_tree_path(docname)):
            return None

        metadata = PageMetadata(
            git_origin_url=self.git_remote.permalink(
                self.git_revision_commit, content_path
            ),
            git_download_url=self.git_remote.permalink(
                self.git_revision_commit, content_path, kind="raw"
            ),
            git_revision_commit=self.git_revision_commit,
            git_revision_time=self.git_revision_time,
        )

        try:
            metadata.git_owner, metadata.git_repo = self.git_remote.repo.split("/")
        except ValueError:
            metadata.git_repo = self.git_remote.repo

        deps = metadata.page_dependencies = [
            PageDependency(type="content", path=content_path),
        ]

        with logger.catch(
            Exception,
            level="TRACE",
            message=f"failed to find .po file for {docname}",
        ):
            gettext_conf = parse_config(self.app.config, GettextConfig)
            gettext_path = (
                self.source_root.joinpath(gettext_conf.locale_dirs[0])
                .joinpath(gettext_conf.language)
                .joinpath("LC_MESSAGES")
                .joinpath(self.env.doc2path(docname, base=False))
                .with_suffix(".po")
                .resolve(strict=True)
                .relative_to(self.git_root)
            )
            deps.append(PageDependency(type="gettext", path=str(gettext_path)))

        for dep in self.env.dependencies.get(docname, set()):
            with logger.catch(
                Exception,
                level="DEBUG",
                message=f"failed to resolve file linked by {docname}",
            ):
                attachment = (
                    self.source_root.joinpath(dep)
                    .resolve(strict=True)
                    .relative_to(self.git_root)
                )
                deps.append(PageDependency(type="attachment", path=str(attachment)))

        def get_timestamp(dep: PageDependency):
            if not self.git_root:
                return

            modified_date: Optional[str] = None
            revision_date: Optional[str] = None
            revision: Optional[str] = None

            with logger.catch(
                Exception,
                level="TRACE",
                message=f"failed to get revision time for {dep.path}",
            ):
                info = subprocess.run(
                    [
                        "git",
                        "log",
                        "-1",
                        r"--format=%cI%%%H",
                        "--",
                        f":/{dep.path}",
                    ],
                    capture_output=True,
                    check=True,
                    text=True,
                ).stdout.strip()
                revision_date, revision = info.split("%")

            with logger.catch(
                Exception,
                level="DEBUG",
                message=f"failed to get modified time for {dep.path}",
            ):
                status = subprocess.run(
                    ["git", "status", "--porcelain", dep.path],
                    capture_output=True,
                    text=True,
                )
                if status.returncode == 0 and status.stdout.strip():
                    mtime = self.git_root.joinpath(dep.path).stat().st_mtime
                    mtime = datetime.fromtimestamp(mtime, timezone.utc)
                    modified_date = mtime.isoformat()

            if modified_date and revision_date:
                if modified_date > revision_date:
                    dep.time = ModifiedTime(time=modified_date)
                else:
                    dep.time = RevisionTime(time=revision_date, commit=revision)
            elif modified_date:
                dep.time = ModifiedTime(time=modified_date)
            elif revision_date:
                dep.time = RevisionTime(time=revision_date, commit=revision)

        for dep in deps:
            get_timestamp(dep)

        for dep in deps:
            match dep.time:
                case RevisionTime(time=time, commit=commit):
                    if (
                        not metadata.git_last_modified_time
                        or metadata.git_last_modified_time < time
                    ):
                        metadata.git_last_modified_time = time
                        metadata.git_last_modified_commit = commit
                case ModifiedTime(time=time):
                    if (
                        not metadata.git_last_modified_time
                        or metadata.git_last_modified_time < time
                    ):
                        metadata.git_last_modified_time = time
                        metadata.git_last_modified_commit = None

        return metadata

    def copy_assets(self) -> None:
        for src, dst in self.static_files.files.items():
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

        manifest = Manifest(sidebar=sidebar, project_name=self.config.project)

        with open(manifest_path, "w+") as f:
            yaml.dump(manifest.model_dump(exclude_none=True, by_alias=True), f)

    def write_buildinfo(self) -> None:
        try:
            self.build_info.dump(self.build_info_path)
        except OSError as exc:
            logger.warning(__("Failed to write build info file: %r"), exc)

    def finish(self):
        self.finish_tasks.add_task(self.copy_assets)
        self.finish_tasks.add_task(self.dump_inventory)
        self.finish_tasks.add_task(self.write_buildinfo)
        self.finish_tasks.add_task(self.write_manifest)

    def cleanup(self) -> None:
        self.ffi.stop()


def is_outdated(output_path: Path, *deps: Path):
    try:
        output_mtime = output_path.stat().st_mtime
    except OSError:
        output_mtime = 0
    for source_path in deps:
        try:
            source_mtime = source_path.stat().st_mtime
        except OSError:
            continue
        if source_mtime > output_mtime:
            return True
    return False
