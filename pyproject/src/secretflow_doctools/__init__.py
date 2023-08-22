from sphinx.application import Sphinx

from sphinx_mdx import setup as setup_mdx


def setup(app: Sphinx):
    setup_mdx(app)
