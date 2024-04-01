from pathlib import Path

from sphinx.application import Sphinx


def test_integrated():
    root = Path(__file__).parent / "docs"
    srcdir = root / "source"
    confdir = root
    outdir = root / "build" / "html"
    doctreedir = root / "build" / ".doctrees"

    app = Sphinx(
        srcdir=str(srcdir.resolve()),
        confdir=str(confdir.resolve()),
        outdir=str(outdir.resolve()),
        doctreedir=str(doctreedir.resolve()),
        buildername="html",
        tags=["html"],
        freshenv=True,
    )

    app.build(force_all=True)
