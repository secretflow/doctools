from typing import cast


def setup(app):
    from loguru import logger
    from sphinx.application import Sphinx

    # from .translator import SphinxJSXTranslator
    from .builder import SphinxJSXBuilder

    logger.disable("sphinx_jsx")

    app = cast(Sphinx, app)

    app.add_builder(SphinxJSXBuilder)
    # app.registry.add_translator("jsx", SphinxJSXTranslator)

    app.tags.add("html")
    app.tags.add("format_html")
    app.tags.add("builder_html")
