import atexit
from contextlib import suppress
from contextvars import ContextVar
from io import BytesIO
from mimetypes import guess_type
from pathlib import Path
from tarfile import TarFile
from typing import BinaryIO, Optional

from flask import Flask, send_file
from pydantic import DirectoryPath, Field
from pydantic_core import to_json
from pydantic_settings import BaseSettings, SettingsConfigDict

from secretflow_doctools.js.cli import get_js_static
from secretflow_doctools.utils.logging import configure_logging


class PreviewServerSettings(BaseSettings):
    output_dir: DirectoryPath = Field(default=...)

    model_config = SettingsConfigDict(env_prefix="doctools_")

    def model_dump_env(self):
        prefix = self.model_config.get("env_prefix")
        assert prefix
        data = self.model_dump(mode="json")
        output: dict[str, str] = {}
        for k, v in data.items():
            if isinstance(v, str):
                output[f"{prefix}{k}"] = v
            else:
                output[f"{prefix}{k}"] = to_json(v).decode()
        return output


def create_app():
    configure_logging()

    def open_tar():
        if app.debug:

            def extract_binary(member: str):
                with get_js_static() as tar:
                    file = tar.extractfile(member)
                    if not file:
                        return None
                    return BytesIO(file.read())

            return extract_binary
        else:

            def extract_stream(member: str):
                try:
                    f = tar.get()
                except LookupError:
                    f = get_js_static()
                    tar.set(f)
                return f.extractfile(member)

            def on_shutdown():
                with suppress(Exception):
                    tar.get().close()

            tar = ContextVar[TarFile]("tar")

            atexit.register(on_shutdown)

            return extract_stream

    env = PreviewServerSettings()

    app = Flask(
        __name__,
        root_path=str(Path.cwd()),
        static_folder=env.output_dir.resolve(),
        static_url_path="/static",
    )

    tar = open_tar()

    def from_tar(path: str):
        file: Optional[BinaryIO]
        try:
            file = tar(path)  # pyright: ignore [reportAssignmentType]
            if not file:
                raise KeyError(path)
        except KeyError:
            path = "index.html"
            file = tar(path)  # pyright: ignore [reportAssignmentType]
            assert file
        return (file, path)

    @app.get("/")
    @app.get("/<path:path>")
    def html(path=""):
        file, path = from_tar(path)
        file: BinaryIO
        mime = guess_type(path)[0]
        return send_file(file, mime)

    app.logger.debug("options: %s", env)

    return app
