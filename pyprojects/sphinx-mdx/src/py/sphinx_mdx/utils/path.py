import os
from pathlib import Path
from typing import Union


def ensure_parent(path: Union[Path, str]) -> None:
    """Ensure that the parent of a path exists."""
    os.makedirs(Path(path).parent, exist_ok=True)
