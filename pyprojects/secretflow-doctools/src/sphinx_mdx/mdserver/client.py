from __future__ import annotations

import platform
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import ContextManager, List, Optional, Tuple
from urllib.parse import quote, urlunsplit

import pexpect
from importlib_resources import files
from loguru import logger
from pexpect.exceptions import ExceptionPexpect
from requests_unixsocket import Session

from .specs.mdast import Root


def find_server_executable() -> Tuple[str, List[str]]:
    import sphinx_mdx

    module_dir = files(sphinx_mdx)
    dist_dir = Path(str(module_dir.joinpath("assets/mdserver/dist")))

    arch = platform.machine().lower()

    if "aarch64" in arch or "arm64" in arch:
        arch_alias = "arm64"
    else:
        arch_alias = "x64"

    system = platform.system()
    if system == "Linux":
        executable_names = ["bin/mdserver-linux", f"bin/mdserver-linux-{arch_alias}"]
    elif system == "Darwin":
        executable_names = ["bin/mdserver-macos", f"bin/mdserver-macos-{arch_alias}"]
    elif system == "Windows":
        executable_names = [
            "bin/mdserver-win.exe",
            f"bin/mdserver-win-{arch_alias}.exe",
        ]
    else:
        executable_names = []

    for name in executable_names:
        executable_path = dist_dir.joinpath(name)
        if executable_path.exists():
            logger.info(f"Using {executable_path.name}")
            return str(executable_path), []

    logger.info("Using Node with script")
    return "node", [str(dist_dir.joinpath("mdserver.cjs"))]


def spawn_server(bind: str) -> pexpect.spawn:
    try:
        executable, args = find_server_executable()
        proc = pexpect.spawn(executable, [*args, bind])
        proc.expect("Listening on")
        return proc
    except ExceptionPexpect as e:
        error_message = f"Failed to start server: {e}\nForgot to install Node?"
        raise RuntimeError(error_message) from e


class MarkdownClient(ContextManager):
    _tempdir: TemporaryDirectory
    _socket_path: str
    _server: pexpect.spawn
    _session: Session

    def __init__(self, server_origin: str | None = None):
        """
        Initialize a MarkdownClient instance.

        You can use it as a context manager:

        ```python
        with MarkdownClient() as server:
            server.parse_markdown(...)
        ```

        or manage the server manually:

        ```python
        server = MarkdownClient()
        server.start()
        server.parse_markdown(...)
        ...
        server.stop()
        ```

        By default, we assume you have node installed, and will automatically launch an
        express server using the bundled JavaScript. If you want to use a different
        script, you can pass the path to it as `script_path`. If you are managing the
        server yourself, you can pass the server origin as `server_origin`, in which
        case no Node process will be spawned.

        :param server_origin: The origin of an external mdserver, such as "http://localhost:3000"
        :type server_origin: Optional[str], optional
        :param script_path: _description_, Path to a JavaScript script file which will\
            be used to launch the server using Node.
        :type script_path: Optional[str], optional
        """
        super().__init__()
        self.server_origin = server_origin

    @property
    def socket_path(self):
        return quote(self._socket_path, safe=())

    def endpoint(self, path: str) -> str:
        if self.server_origin:
            return self.server_origin.rstrip("/") + path
        return urlunsplit(("http+unix", self.socket_path, path, "", ""))

    def markdown_to_tree(self, document: str) -> Root:
        res = self._session.post(
            self.endpoint("/parse/markdown"),
            data=document.encode("utf-8"),
            headers={"Content-Type": "text/plain"},
        )
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.json()

    def html_to_tree(self, document: str) -> Root:
        res = self._session.post(
            self.endpoint("/parse/html"),
            data=document.encode("utf-8"),
            headers={"Content-Type": "text/plain"},
        )
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.json()

    def tree_to_markdown(self, ast: Root) -> str:
        res = self._session.post(
            self.endpoint("/stringify/markdown"),
            json=ast,
        )
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.text

    def start(self):
        self._session = Session()
        if self.server_origin:
            return
        self._tempdir = TemporaryDirectory()
        self._socket_path = str(Path(self._tempdir.name).joinpath("socket").absolute())
        self._server = spawn_server(self._socket_path)

    def stop(self):
        if not self.server_origin:
            self._server.terminate()
            self._tempdir.cleanup()
        self._session.close()

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, __exc_type, __exc_value, __traceback) -> Optional[bool]:
        self.stop()
        return super().__exit__(__exc_type, __exc_value, __traceback)
