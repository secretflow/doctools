import os
import sys
import tarfile
from pathlib import Path
from typing import Any

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

GIT_ROOT = Path(__file__).joinpath("../..").resolve()
PKG_ROOT = Path(__file__).joinpath("../../src/py/secretflow_doctools").resolve()


class JsBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        build_static()


def build_static():
    web_dist = GIT_ROOT.joinpath("dist/web")

    with tarfile.open(PKG_ROOT.joinpath("js/web.tar"), "w") as tar:
        for parent, _, files in os.walk(web_dist):
            for name in files:
                full = Path(parent).joinpath(name)
                name = full.relative_to(web_dist)
                tar.add(full, name)
                print(f"tar {name}", file=sys.stderr)


if __name__ == "__main__":
    build_static()
