from loguru import logger
from sphinx.application import Sphinx

from .builder import MDXBuilder
from .options import setup_config
from .translator import MDXTranslator

logger.disable("sphinx_mdx")


def setup(app: Sphinx):
    app.add_builder(MDXBuilder)
    app.registry.add_translator("mdx", MDXTranslator)

    # Enable HTML output
    app.tags.add("html")
    app.tags.add("format_html")
    app.tags.add("builder_html")

    # Setup config keys
    setup_config(app)
