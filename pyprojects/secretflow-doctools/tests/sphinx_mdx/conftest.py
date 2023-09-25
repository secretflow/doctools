# https://github.com/modelmat/sphinxcontrib-drawio/blob/main/tests/conftest.py

import pytest
from sphinx.application import Sphinx
from sphinx.testing.path import path


@pytest.fixture(scope="module")
def rootdir(request):
    specified_path = getattr(request, "param", None)
    if specified_path:
        return specified_path
    test_name = request.module.__name__.split(".")[-1]
    return path(__file__).parent.abspath() / test_name


@pytest.fixture(scope="function")
def content(app: Sphinx):
    app.build()
    return app
