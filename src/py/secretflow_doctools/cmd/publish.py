import json
import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Optional
from urllib.parse import urlsplit

import requests
from loguru import logger
from pydantic import BaseModel, Field, SecretStr
from pydantic_settings import BaseSettings, SettingsConfigDict

from secretflow_doctools.cmd.util import fatal_on_missing_env_vars
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.utils.subprocess import fatal_on_subprocess_error
from secretflow_doctools.vcs import HeadRef, git_describe


def publish(name: str, index_js: str, registry: str, tag: Optional[str]):
    dry_run = os.getenv("DRY_RUN") != "0"

    porcelain(dry_run=dry_run)

    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d%H%M%S%f")
    revision = git_describe()

    match revision.src:
        case HeadRef():
            logger.error(
                _(
                    "refusing to publish: the current commit is not:\n"
                    "  - tagged, or\n"
                    "  - on a release branch containing a tagged commit, or\n"
                    "  - main or master"
                )
            )
            exit(1)

    registry_url = urlsplit(registry)

    package_version = f"0.1.0-g{revision.sha[:7]}-b{timestamp}"
    package_tag = tag if tag else f"gh-{revision.ref}"
    package_json = {
        "name": name,
        "version": package_version,
        "author": "SecretFlow <secretflow-contact@service.alipay.com>",
        "description": "SecretFlow documentation build artifacts",
        "publishConfig": {
            "access": "public",
            "registry": registry_url.geturl(),
        },
        "exports": {
            ".": {
                "import": "./dist/index.js",
                "default": "./dist/index.js",
            },
        },
        "files": ["index.js", "dist"],
        "x-secretflow-refs": [],
    }

    published = requests.get(registry_url._replace(path=f"/{name}").geturl()).text
    published = PublishedPackage.model_validate_json(published)
    published = [t for t in published.dist_tags if t.startswith("gh-")]

    package_json["x-secretflow-refs"].extend(published)

    with fatal_on_missing_env_vars(Credentials):
        credentials = Credentials()

    with TemporaryDirectory() as package_root:
        package_root = Path(package_root)

        shutil.copytree(Path(index_js).parent, package_root.joinpath("dist"))
        with open(package_root.joinpath("package.json"), "w+") as f:
            json.dump(package_json, f)

        with open(package_root.joinpath(".npmrc"), "w+") as f:
            f.write(f"//{registry_url.netloc}/:_authToken = ${{NPM_TOKEN}}\n")

        with fatal_on_subprocess_error(
            "npm",
            "publish",
            *["--dry-run"] if dry_run else [],
            "--tag",
            package_tag,
            ".",
        ) as cmd:
            subprocess.run(
                cmd,
                env={
                    **os.environ,
                    "NPM_TOKEN": credentials.npm_token.get_secret_value(),
                },
                cwd=package_root,
                stdout=None,
                stderr=None,
                text=True,
            ).check_returncode()


class Credentials(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="doctools_publish_")

    npm_token: SecretStr = Field(default=...)


class PublishedPackage(BaseModel):
    dist_tags: dict[str, str] = Field(alias="dist-tags")


def porcelain(dry_run: bool):
    with fatal_on_subprocess_error("git", "status", "--porcelain") as cmd:
        status = subprocess.run(
            cmd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=True,
        )

    if not dry_run and status.stdout:
        raise ValueError(_("refusing to publish: git has uncommitted changes"))
