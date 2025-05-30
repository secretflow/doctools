import shutil
from pathlib import Path
from subprocess import check_call
from tempfile import TemporaryDirectory
from typing import Any, Callable

from hatchling.builders.plugin.interface import IncludedFile
from hatchling.builders.utils import normalize_artifact_permissions, replace_file
from hatchling.builders.wheel import (
    RecordFile,
    WheelArchive,
    WheelBuilder,
)

GIT_ROOT = Path(__file__).joinpath("../..").resolve()
PKG_ROOT = Path(__file__).joinpath("../../src/py/secretflow_doctools").resolve()

TARGETS = {
    "macosx_11_0_arm64": "aarch64-apple-darwin",
    "macosx_10_9_x86_64": "x86_64-apple-darwin",
    "win_amd64": "x86_64-pc-windows-msvc",
    "manylinux_2_18_x86_64": "x86_64-unknown-linux-gnu",
    "manylinux_2_18_aarch64": "aarch64-unknown-linux-gnu",
}


class WheelBuilderWithDenort(WheelBuilder):
    def get_version_api(self) -> dict[str, Callable[..., Any]]:
        return {platform: self.build_for(platform) for platform in TARGETS}

    def get_default_versions(self) -> list[str]:
        return [*TARGETS.keys()]

    def build_for(self, platform: str):
        def build(*args, **kwargs):
            return self.build_target(platform, *args, **kwargs)

        return build

    def build_target(
        self,
        platform: str,
        directory: str,
        **build_data,
    ) -> str:
        triple = TARGETS[platform]

        tag = f"py3-none-{platform}"

        with TemporaryDirectory() as temp:
            shutil.copyfile(
                PKG_ROOT.joinpath("js/cli.js"),
                Path(temp).joinpath("cli.js"),
            )

            check_call(
                [
                    "deno",
                    "compile",
                    "--allow-all",
                    "--no-check",
                    "--target",
                    triple,
                    "--output",
                    "cli",
                    "cli.js",
                ],
                cwd=temp,
            )

            outfile = Path(temp).joinpath("cli")

            if "windows-msvc" in triple:
                outfile = outfile.with_suffix(".exe")

            with (
                WheelArchive(
                    self.artifact_project_id,
                    reproducible=self.config.reproducible,
                ) as archive,
                RecordFile() as records,
            ):
                for included_file in self.recurse_included_files():
                    record = archive.add_file(included_file)
                    records.write(record)

                with open(outfile, "rb") as f:
                    bin_relpath = Path("secretflow_doctools/js")
                    bin_relpath = bin_relpath.joinpath(outfile.name)
                    bin_relpath = str(bin_relpath)
                    bin_name = "secretflow-doctools-js-cli"
                    if "windows-msvc" in triple:
                        bin_name += ".exe"
                    included_file = IncludedFile(str(outfile), bin_relpath, bin_name)
                    record = archive.write_shared_script(included_file, f.read())
                    records.write(record)

                self.write_data(
                    archive,
                    records,
                    {**build_data, "pure_python": False, "tag": tag},
                    build_data["dependencies"],
                )

                records.write((f"{archive.metadata_directory}/RECORD", "", ""))
                archive.write_metadata("RECORD", records.construct())

        target = Path(directory).joinpath(f"{self.artifact_project_id}-{tag}.whl")
        target = str(target)

        replace_file(archive.path, target)
        normalize_artifact_permissions(target)

        return target


def get_builder():
    return WheelBuilderWithDenort
