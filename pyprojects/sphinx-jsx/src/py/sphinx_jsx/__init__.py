from loguru import logger
from sphinx.application import Sphinx

logger.disable("sphinx_jsx")


def setup(app: Sphinx):
    from .builder import SphinxJSXBuilder
    from .options import setup_config
    from .syntax import html, math, rst
    from .translator import SphinxJSXTranslator

    app.add_builder(SphinxJSXBuilder)
    app.registry.add_translator("jsx", SphinxJSXTranslator)

    app.tags.add("html")
    app.tags.add("format_html")
    app.tags.add("builder_html")

    rst.setup(app)
    math.setup(app)
    html.setup(app)

    setup_config(app)
