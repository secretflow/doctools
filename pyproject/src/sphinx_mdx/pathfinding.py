from __future__ import annotations

import os.path
from pathlib import Path
from typing import Dict, Set
from urllib.parse import urlsplit, urlunsplit

from docutils import nodes
from sphinx import addnodes
from sphinx.builders import Builder

from .options import MDXOptions


class Pathfinder:
    """Pathfinding helper functions."""

    def __init__(self, builder: Builder, options: MDXOptions) -> None:
        self.builder = builder
        self.options = options

    @property
    def path_normalizer(self):
        return self.options.mdx_output_path_normalizer or (lambda x: x)

    @property
    def output_root(self) -> Path:
        return Path(self.builder.outdir)

    def get_target_uri(self, docname: str) -> str:
        normalized = self.path_normalizer(docname)
        return normalized

    def get_output_path(self, docname: str) -> Path:
        """Resolve a documentation's output file path."""
        name = self.builder.get_target_uri(docname)
        output_path = self.output_root.joinpath(name)
        output_path = self.append_suffix(output_path)
        return output_path

    def get_output_path_from_refuri(self, refuri: str) -> str:
        """Resolve the path to another doc's output file path given the refuri."""
        parsed = urlsplit(refuri)
        if parsed.path:
            # if path is empty then most likely this is a reference to a section
            path = str(self.append_suffix(Path(parsed.path)))
            parsed = parsed._replace(path=path)
        return urlunsplit(parsed)

    def append_suffix(self, path: Path) -> Path:
        existing_suffix = "".join(path.suffixes)
        suffix = f"{existing_suffix}{self.options.mdx_output_file_suffix}"
        return path.with_suffix(suffix)

    @staticmethod
    def is_external_url(uri: str) -> bool:
        """
        Determine whether a URI is an external URL.

        This is a very simple check that only checks for the presence of a scheme.
        """
        try:
            parsed = urlsplit(uri)
        except ValueError:
            # malformed
            return True
        return parsed.scheme not in ("", "file")


class StaticFiles:
    """
    Static files collection.

    StaticFiles records all static files (images, downloadable files, etc.) that are
    encountered throughout the build process and determines their eventual location in
    the build output. It also generates relative paths to these files from the
    referenced doc page to be used in the doc's text output.
    """

    def __init__(self, builder: Builder, options: MDXOptions) -> None:
        self.builder = builder
        self.options = options

        self.pathfinder = Pathfinder(builder, options)

        self.files: Dict[Path, Path] = {}
        self.outputs: Set[Path] = set()

    @property
    def source_root(self) -> Path:
        return Path(self.builder.srcdir)

    @property
    def output_root(self) -> Path:
        return Path(self.builder.outdir)

    @property
    def static_root(self) -> Path:
        return self.output_root / self.options.mdx_assets_output_dir

    def get_unique_output_path(self, filepath: Path) -> Path:
        output_path = self.static_root.joinpath(filepath.name)
        counter = 0
        while output_path in self.outputs:
            counter += 1
            new_name = f"{output_path.stem}-{counter}{output_path.suffix}"
            output_path = output_path.with_name(new_name)
        return output_path

    def include_file(self, filepath: Path) -> Path:
        filepath = filepath.resolve()
        existing = self.files.get(filepath)
        if existing is not None:
            return existing
        output = self.get_unique_output_path(filepath)
        self.files[filepath] = output
        self.outputs.add(output)
        return output

    def add_image(self, docname: str, node: nodes.image) -> str:
        """
        Include an image and return a relative path to it.

        This method is idempotent.
        """

        uri = node["uri"]

        if self.pathfinder.is_external_url(uri):
            # not a local file
            return uri

        # uri will be the path to the image relative to srcdir
        source_path = self.source_root.joinpath(uri).resolve()
        output_path = self.include_file(source_path)

        doc_output_path = self.pathfinder.get_output_path(docname)
        return os.path.relpath(output_path, start=doc_output_path.parent)

    def add_downloadable_file(
        self,
        docname: str,
        node: addnodes.download_reference,
    ) -> str:
        """
        Include a downloadable file and return the relative path to it.

        This method is idempotent.
        """

        if "reftarget" not in node:
            # not a local file
            return node["refuri"]

        # reftarget will be relative to doc source
        doc_source_path = Path(self.builder.env.doc2path(docname))

        source_path = doc_source_path.parent.joinpath(node["reftarget"]).resolve()
        output_path = self.include_file(source_path)

        doc_output_path = self.pathfinder.get_output_path(docname)
        return os.path.relpath(output_path, start=doc_output_path.parent)
