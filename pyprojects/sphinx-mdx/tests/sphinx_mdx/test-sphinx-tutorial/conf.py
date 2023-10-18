import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.absolute()))

extensions = [
    "sphinx_mdx",
    "sphinx.ext.intersphinx",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
]

exclude_patterns = ["_build"]

smartquotes = False

intersphinx_mapping = {"python": ("https://docs.python.org/3.11", None)}
