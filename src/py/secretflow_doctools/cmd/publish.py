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

from secretflow_doctools.cmd.util import require_env_vars
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.utils.subprocess import fatal_on_subprocess_error
from secretflow_doctools.vcs import HeadRef, git_describe


def publish(name: str, index_js: str, registry: str, tag: Optional[str]):
    dry_run = os.getenv("DRY_RUN") != "0"

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
            raise SystemExit(1)

    registry_url = urlsplit(registry)
    logger.info(_("using registry: {reg}"), reg=registry_url.netloc)

    package_version = f"0.1.0-g{revision.sha[:7]}-b{timestamp}"
    package_tag = tag if tag else f"gh-{revision.ref}"

    logger.info(_("name:    {name}"), name=repr(name))
    logger.info(_("tags:    {tag}, latest"), tag=repr(package_tag))
    logger.info(_("version: {version}"), version=repr(package_version))

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
        "x-secretflow-refs": [revision.ref],
    }

    logger.info(_("fetching existing refs"))

    with logger.catch(
        Exception,
        level="CRITICAL",
        message=_("failed to determine existing refs"),
        onerror=lambda _: exit(1),
    ):
        published = []
        res = requests.get(registry_url._replace(path=f"/{name}").geturl())
        match res.status_code:
            case 200:
                pkg = Package200.model_validate_json(res.text)
                published = [t[3:] for t in pkg.dist_tags if t.startswith("gh-")]
                logger.info(_("found refs: {v}"), v=", ".join(published))
            case 404:
                logger.warning(_("received 404, assuming a new package"))
            case status:
                raise ValueError(f"unexpected {status}: {res.text}")

    package_json["x-secretflow-refs"].extend(published)

    logger.debug(json.dumps(package_json, indent=2))

    if dirty := porcelain():
        logger.warning(
            _("git has uncommitted changes:\n{dirty}"),
            dirty=dirty.strip("\n"),
        )
        if not dry_run:
            logger.critical(_("refusing to publish"))
            raise SystemExit(1)

    with TemporaryDirectory() as package_root:
        package_root = Path(package_root)
        logger.debug(_("setting up package at {dir}"), dir=package_root)

        shutil.copytree(Path(index_js).parent, package_root.joinpath("dist"))
        with open(package_root.joinpath("package.json"), "w+") as f:
            json.dump(package_json, f)

        with open(package_root.joinpath(".npmrc"), "w+") as f:
            f.write(f"//{registry_url.netloc}/:_authToken = ${{NPM_TOKEN}}\n")

        def getenv():
            if dry_run:
                return {**os.environ}
            else:
                token = require_env_vars(Credentials).npm_token.get_secret_value()
                return {
                    **os.environ,
                    "NPM_TOKEN": token,
                }

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
                env=getenv(),
                cwd=package_root,
                stdout=None,
                stderr=None,
                text=True,
            ).check_returncode()

        if not dry_run:
            with fatal_on_subprocess_error(
                "npm",
                "dist-tag",
                "add",
                f"{name}@{package_version}",
                "latest",
            ) as cmd:
                subprocess.run(
                    cmd,
                    env=getenv(),
                    cwd=package_root,
                    stdout=None,
                    stderr=None,
                    text=True,
                ).check_returncode()


class Credentials(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="doctools_publish_")

    npm_token: SecretStr = Field(default=...)


class Package200(BaseModel):
    dist_tags: dict[str, str] = Field(alias="dist-tags")


def porcelain():
    with fatal_on_subprocess_error("git", "status", "--porcelain") as cmd:
        status = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            check=True,
        )

    return status.stdout
