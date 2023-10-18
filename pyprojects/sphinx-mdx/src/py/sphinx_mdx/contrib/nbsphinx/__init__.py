from sphinx.application import Sphinx

from sphinx_mdx.utils.sphinx import override_handlers

from . import handlers, nodes, transforms


def setup(app: Sphinx):
    from nbsphinx import (  # pyright: ignore[reportMissingImports]
        CodeAreaNode,
        FancyOutputNode,
    )

    app.add_post_transform(transforms.NbSphinxCellTransform)

    override_handlers(
        app,
        nodes.nbsphinx_container,
        handlers.visit_nbsphinx_container,
    )
    override_handlers(
        app,
        CodeAreaNode,
        handlers.visit_CodeAreaNode,
        handlers.depart_CodeAreaNode,
    )
    override_handlers(
        app,
        FancyOutputNode,
        handlers.visit_FancyOutputNode,
        handlers.depart_FancyOutputNode,
    )
