import re
import subprocess
from typing import Literal, Optional, Union
from urllib.parse import urlsplit, urlunsplit

from loguru import logger
from pydantic import BaseModel

from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.utils.subprocess import fatal_on_subprocess_error, running


class RemoteVCS(BaseModel):
    hostname: str
    path: str
    repo: str

    @property
    def name(self) -> str:
        *_, name = self.repo.split("/")
        return name

    def permalink(self, revision: str, path: str, kind="blob"):
        prefix = self.path.removesuffix("/").removesuffix(".git")
        return f"https://{self.hostname}{prefix}/{kind}/{revision}/{path}"


def guess_remote_vcs(origin: str) -> Optional[RemoteVCS]:
    try:
        url = urlsplit(origin)
        if not url.hostname:
            raise ValueError
    except ValueError:
        pass
    else:
        hostname = url.hostname
        path = url.path
        try:
            *_, owner, repo = filter(None, path.split("/"))
            repo = f"{owner}/{repo}"
        except ValueError:
            repo = path.split("/")[-1]
        if repo.endswith(".git"):
            repo = repo[:-4]
        return RemoteVCS(hostname=hostname, path=path, repo=repo)
    try:
        auth, hostpath = origin.split("@", 1)
        sep = hostpath.rfind(":")
        if sep == -1:
            raise ValueError
        hostname = hostpath[:sep]
        path = hostpath[sep + 1 :]
        url = urlunsplit(("ssh", hostname, path, "", ""))
        return guess_remote_vcs(url)
    except ValueError:
        return None


def git_origin(remote="origin") -> Optional[str]:
    with logger.catch(
        level="WARNING",
        message=_("failed to get url for remote {remote}").format(remote=repr(remote)),
    ):
        return (
            subprocess.run(
                ["git", "config", "--get", f"remote.{remote}.url"],
                capture_output=True,
                check=True,
            )
            .stdout.decode("utf-8")
            .strip()
        )


class PreciseRef(BaseModel):
    kind: Literal["precise"] = "precise"


class NearestRef(BaseModel):
    kind: Literal["nearest"] = "nearest"
    branch: str


class MainRef(BaseModel):
    kind: Literal["main"] = "main"


class HeadRef(BaseModel):
    kind: Literal["head"] = "head"


DescribeSource = Union[PreciseRef, NearestRef, MainRef, HeadRef]


class GitDescribe(BaseModel):
    sha: str
    ref: str
    src: DescribeSource


def git_describe() -> GitDescribe:
    with fatal_on_subprocess_error("git", "rev-parse", "--abbrev-ref", "HEAD") as cmd:
        head = subprocess.run(
            cmd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        ).stdout.strip()

    with fatal_on_subprocess_error("git", "rev-parse", "HEAD") as cmd:
        sha = subprocess.run(
            cmd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=True,
        ).stdout.strip()

    ref = subprocess.run(
        running("git", "describe", "--tags", "--exact-match"),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )

    if ref.returncode == 0:
        return GitDescribe(src=PreciseRef(), sha=sha, ref=ref.stdout.strip())

    ref = subprocess.run(
        running("git", "describe", "--tags", "--long", "--first-parent"),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )

    if ref.returncode == 0:
        if not (ref := RE_GIT_DESCRIBE.match(ref.stdout.strip())):
            raise ValueError(
                _("unexpected git describe output: {output}").format(output=ref)
            )

        if head.startswith("release/"):
            return GitDescribe(
                src=NearestRef(branch=head),
                sha=sha,
                ref=ref.groupdict()["tag"],
            )

    match head:
        case "master" | "main":
            return GitDescribe(src=MainRef(), sha=sha, ref=head)
        case _:
            return GitDescribe(src=HeadRef(), sha=sha, ref=head)


RE_GIT_DESCRIBE = re.compile(r"^(?P<tag>.+)-(?P<distance>\d+)-g(?P<sha>[0-9a-f]+)$")
