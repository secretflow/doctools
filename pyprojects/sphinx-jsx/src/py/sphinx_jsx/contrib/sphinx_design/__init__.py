from sphinx.application import Sphinx

from sphinx_jsx.utils.sphinx import override_handlers

from . import elements, transforms, translator


def setup(app: Sphinx):
    app.add_post_transform(transforms.SphinxDesignPostTransform)

    override_handlers(
        app,
        elements.container__sphinx_design_card,
        translator.visit,
        translator.depart,
    )
