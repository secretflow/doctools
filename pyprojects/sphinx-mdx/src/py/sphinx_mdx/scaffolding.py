import os
import shutil
import tempfile
from pathlib import Path
from typing import Dict
from urllib.parse import urlsplit, urlunsplit

from docutils import nodes
from pydantic import ValidationError, validate_arguments
from sphinx import addnodes

from ._lib import compile_mdx
from .mdx import Root
from .options import BuildEnvironment, PackageJSON, ScaffoldingOptions
from .utils.logging import get_logger
from .utils.path import ensure_parent

logger = get_logger(__name__)


class Scaffolding:
    @classmethod
    @validate_arguments
    def default(cls, env: BuildEnvironment):
        package = PackageJSON(
            name="sphinx-mdx",
            version="0.0.0",
            sphinx=ScaffoldingOptions(
                emits="js",
                output_dir=".",
                assets_dir="_assets",
            ),
        )
        package_json = env.outdir / "package.json"
        package_json.unlink(missing_ok=True)
        ensure_parent(package_json)
        package_json.write_text(package.json(indent=2))
        return cls(env, package)

    @classmethod
    @validate_arguments
    def create(cls, env: BuildEnvironment):
        if not env.mdx_scaffolding:
            return cls.default(env)

        path = Path(env.mdx_scaffolding)

        with tempfile.TemporaryDirectory() as tmpdir:
            if path.is_dir():
                staging = path
            else:
                staging = Path(tmpdir)

                try:
                    shutil.unpack_archive(str(path), extract_dir=staging)
                except shutil.ReadError as e:
                    raise ValueError(f"Unsupported scaffolding {path}") from e

            try:
                with open(staging / "package.json") as f:
                    package = PackageJSON.parse_raw(f.read())
            except FileNotFoundError as e:
                raise ValueError(f"package.json not found in {path}") from e
            except ValidationError as e:
                raise ValueError(f"Invalid scaffolding: {e}") from e
            except Exception as e:
                raise ValueError(f"Unsupported scaffolding {e}") from e

            if package.files:
                for file in [*package.files, "package.json"]:
                    src_path = staging / file
                    if not src_path.exists():
                        continue
                    dst_path = env.outdir / file
                    ensure_parent(dst_path)
                    if src_path.is_dir():
                        shutil.copytree(src_path, dst_path, dirs_exist_ok=True)
                    else:
                        shutil.copy(src_path, dst_path)
            else:
                shutil.copytree(staging, env.outdir, dirs_exist_ok=True)

            return cls(env, package)

    def __init__(self, env: BuildEnvironment, package: PackageJSON):
        self.env = env
        self.package = package

        self.asset_map: Dict[Path, Path] = {}

    @property
    def path_normalizer(self):
        return self.env.mdx_output_path_normalizer or (lambda x: x)

    @property
    def source_root(self) -> Path:
        return self.env.srcdir

    @property
    def output_root(self) -> Path:
        return self.env.outdir / self.package.sphinx.output_dir

    @property
    def static_root(self) -> Path:
        return self.output_root / self.package.sphinx.assets_dir

    def get_target_uri(self, docname: str) -> str:
        normalized = self.path_normalizer(docname)
        return normalized

    def get_output_path(self, docname: str) -> Path:
        """Resolve a documentation's output file path."""
        name = self.get_target_uri(docname)
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
        if self.package.sphinx.emits == "js":
            suffix = f"{existing_suffix}.jsx"
        else:
            suffix = f"{existing_suffix}.mdx"
        return path.with_suffix(suffix)

    def get_unique_output_path(self, filepath: Path) -> Path:
        output_path = self.static_root.joinpath(filepath.name)
        counter = 0
        asset_outputs = set(self.asset_map.values())
        while output_path in asset_outputs:
            counter += 1
            new_name = f"{output_path.stem}-{counter}{output_path.suffix}"
            output_path = output_path.with_name(new_name)
        return output_path

    def include_file(self, filepath: Path) -> Path:
        filepath = filepath.resolve()

        existing = self.asset_map.get(filepath)
        if existing is not None:
            return existing

        output_path = self.static_root.joinpath(filepath.name)
        counter = 0
        asset_outputs = set(self.asset_map.values())

        while output_path in asset_outputs:
            counter += 1
            new_name = f"{output_path.stem}-{counter}{output_path.suffix}"
            output_path = output_path.with_name(new_name)

        self.asset_map[filepath] = output_path
        return output_path

    def add_image(self, docname: str, node: nodes.image) -> str:
        """
        Include an image and return a relative path to it.

        This method is idempotent.
        """

        uri = node["uri"]

        if self.is_external_url(uri):
            # not a local file
            return uri

        # uri will be the path to the image relative to srcdir
        source_path = self.source_root.joinpath(uri).resolve()
        output_path = self.include_file(source_path)

        doc_output_path = self.get_output_path(docname)
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
        doc_source_path = Path(self.env.doc2path(docname))

        source_path = doc_source_path.parent.joinpath(node["reftarget"]).resolve()
        output_path = self.include_file(source_path)

        doc_output_path = self.get_output_path(docname)
        return os.path.relpath(output_path, start=doc_output_path.parent)

    def add_doc(self, docname: str, doc: Root):
        content = str(doc)

        if self.package.sphinx.emits == "js":
            content = compile_mdx(content)

        output_path = self.get_output_path(docname)
        ensure_parent(output_path)
        output_path.write_text(content)

    def copy_assets(self) -> None:
        for src, dst in self.asset_map.items():
            ensure_parent(dst)
            shutil.copy(src, dst)

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
