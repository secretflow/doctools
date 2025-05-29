def setup(_app):
    from sphinx.application import Sphinx

    from secretflow_doctools.builder import MdxBuilder
    from secretflow_doctools.i18n.builder import SwaggerGettextBuilder
    from secretflow_doctools.options import MdxConfig, setup_config

    app: Sphinx = _app

    app.add_builder(MdxBuilder)
    app.add_builder(SwaggerGettextBuilder)

    # Enable HTML output
    app.tags.add("html")
    app.tags.add("format_html")
    app.tags.add("builder_html")

    setup_config(app, MdxConfig)
