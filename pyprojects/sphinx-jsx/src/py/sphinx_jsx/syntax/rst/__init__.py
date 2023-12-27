from sphinx.application import Sphinx

from . import elements as e
from . import transforms


def setup(app: Sphinx):
    app.add_post_transform(transforms.TypePromotionTransform)


__all__ = ["e", "transforms"]
