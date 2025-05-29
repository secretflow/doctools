from __future__ import annotations

import atexit
import signal
import socket
from tempfile import TemporaryDirectory
from typing import ContextManager, Optional

from pexpect.popen_spawn import PopenSpawn
from requests import Session

from secretflow_doctools.js.cli import get_js_binary

from .specs.mdast import Root


class MarkdownClient(ContextManager):
    _tempdir: TemporaryDirectory
    _server: PopenSpawn
    _listen: str
    _session: Session

    def endpoint(self, path: str) -> str:
        return f"http://localhost:{self._listen}{path}"

    def markdown_to_tree(self, document: str) -> Root:
        res = self._session.post(
            self.endpoint("/markdown/parse"),
            data=document.encode("utf-8"),
            headers={"Content-Type": "text/plain"},
        )
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.json()

    def html_to_tree(self, document: str) -> Root:
        res = self._session.post(
            self.endpoint("/html/parse"),
            data=document.encode("utf-8"),
            headers={"Content-Type": "text/plain"},
        )
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.json()

    def tree_to_markdown(self, ast: Root) -> str:
        res = self._session.post(self.endpoint("/markdown/stringify"), json=ast)
        if res.status_code != 200:
            raise ValueError(res.text)
        return res.text

    def start(self):
        self._session = Session()
        self._tempdir = TemporaryDirectory()
        self._listen = str(find_free_port_with_race_condition())
        executable, args = get_js_binary()
        cmd = [executable, *args, "ffi", "-p", self._listen]
        self._server = PopenSpawn(cmd)
        try:
            self._server.expect("Listening on")
        except BaseException:
            self.stop()

        @atexit.register
        def cleanup(*args, **kwargs):
            try:
                self._server.kill(signal.SIGINT)
            except Exception:
                pass

    def stop(self):
        self._server.kill(signal.SIGINT)
        self._session.close()

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, __exc_type, __exc_value, __traceback) -> Optional[bool]:
        self.stop()


def find_free_port_with_race_condition():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("localhost", 0))
        return s.getsockname()[1]
