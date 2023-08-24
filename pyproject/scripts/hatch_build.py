import os
from typing import Any, Callable, Dict, List

from hatchling.builders.utils import replace_file
from hatchling.builders.wheel import (
    RecordFile,
    WheelArchive,
    WheelBuilder,
    WheelBuilderConfig,
)

SUPPORTED_PLATFORMS: List[str] = [
    "win_amd64",
    "manylinux_2_17_x86_64",
    "macosx_10_9_x86_64",
]


class VercelPackageWheelBuilder(WheelBuilder):
    def get_version_api(self) -> Dict[str, Callable]:
        return {
            platform: self.create_builder(platform) for platform in SUPPORTED_PLATFORMS
        }

    def get_default_versions(self) -> List[str]:
        return SUPPORTED_PLATFORMS

    def create_builder(self, platform_tag: str) -> Callable:
        if platform_tag.startswith("manylinux"):
            suffix = "-linux"
        elif platform_tag.startswith("macosx"):
            suffix = "-macos"
        elif platform_tag.startswith("win"):
            suffix = "-win.exe"
        else:
            raise ValueError(f"Unsupported platform tag: {platform_tag}")

        def build(directory: str, **build_data: Any) -> str:
            build_data["tag"] = f"py3-none-{platform_tag}"
            build_data["pure_python"] = False

            config: WheelBuilderConfig = self.config

            # The following comes from super().build_standard

            with WheelArchive(
                self.artifact_project_id,
                reproducible=self.config.reproducible,
            ) as archive, RecordFile() as records:
                for included_file in self.recurse_included_files():
                    if config.path_is_artifact(
                        included_file.relative_path
                    ) and not included_file.relative_path.endswith(suffix):
                        continue
                    record = archive.add_file(included_file)
                    records.write(record)

                self.write_data(
                    archive,
                    records,
                    build_data,
                    build_data["dependencies"],
                )

                records.write((f"{archive.metadata_directory}/RECORD", "", ""))
                archive.write_metadata("RECORD", records.construct())

            target = os.path.join(
                directory,
                f"{self.artifact_project_id}-{build_data['tag']}.whl",
            )

            replace_file(archive.path, target)
            return target

        return build


def get_builder():
    return VercelPackageWheelBuilder
