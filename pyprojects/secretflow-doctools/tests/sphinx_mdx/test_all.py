from pathlib import Path

from sphinx.application import Sphinx

from .utils import parametrize_test_projects


@parametrize_test_projects(Path(__file__).parent)
def test_all(content: Sphinx, expect_file: Path):
    srcdir = Path(content.srcdir)
    outdir = Path(content.outdir)

    expected = srcdir.joinpath("expect").joinpath(expect_file)
    actual = outdir.joinpath(expect_file)

    assert actual.exists()

    try:
        assert actual.read_text() == expected.read_text()
    except UnicodeDecodeError:
        assert actual.read_bytes() == expected.read_bytes()
