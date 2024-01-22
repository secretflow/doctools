from typing import cast


def setup(app):
    from loguru import logger
    from sphinx.application import Sphinx

    from .options import setup_config
    from .syntax import html, math, rst

    # from .translator import SphinxJSXTranslator

    logger.disable("sphinx_jsx")

    app = cast(Sphinx, app)

    # app.add_builder(SphinxJSXBuilder)
    # app.registry.add_translator("jsx", SphinxJSXTranslator)

    app.tags.add("html")
    app.tags.add("format_html")
    app.tags.add("builder_html")

    rst.setup(app)
    math.setup(app)
    html.setup(app)

    setup_config(app)
