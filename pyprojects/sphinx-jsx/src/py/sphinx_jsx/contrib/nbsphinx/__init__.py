from sphinx.application import Sphinx

from sphinx_jsx.utils.sphinx import override_handlers

from . import elements, transforms, translator


def setup(app: Sphinx):
    from nbsphinx import CodeAreaNode, FancyOutputNode

    app.add_post_transform(transforms.NbSphinxCellTransform)

    override_handlers(
        app,
        elements.container__nbsphinx,
        translator.visit_container,
        translator.depart_container,
    )
    override_handlers(
        app,
        CodeAreaNode,
        translator.visit_code_area,
        translator.depart_code_area,
    )
    override_handlers(
        app,
        FancyOutputNode,
        translator.visit_code_area,
        translator.depart_code_area,
    )
