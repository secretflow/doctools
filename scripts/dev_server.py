import shutil
import signal
import subprocess
import sys
from contextlib import suppress
from pathlib import Path

dirname = Path(__file__).parent

if __name__ == "__main__":
    server = subprocess.Popen(
        [
            sys.executable,
            "-m",
            "secretflow_doctools",
            "preview",
        ],
        stdin=subprocess.DEVNULL,
    )

    client = subprocess.Popen(
        [
            shutil.which("node") or "node",
            dirname.joinpath("../node_modules/vite/bin/vite.js"),
            "--clearScreen=false",
        ]
    )

    with suppress(KeyboardInterrupt):
        client.wait()

    server.send_signal(signal.SIGTERM)
    server.wait()
