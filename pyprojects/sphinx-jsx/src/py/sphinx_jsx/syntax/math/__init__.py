from sphinx.application import Sphinx

from . import elements as e


def setup(app: Sphinx):
    from .transforms import MathRenderingTransform

    app.add_post_transform(MathRenderingTransform)


__all__ = ["e"]
