from pathlib import Path
from typing import List, Tuple

import pytest
from sphinx.testing.path import path


def parametrize_test_projects(root: Path):
    """Parametrize a test function with all test projects in a directory."""
    # rootdir contains all test projects
    rootdir = Path(root.absolute())

    # find all test projects according to Sphinx fixture rules
    projects = [p.name[5:] for p in root.glob("test-*")]

    parameters: List[Tuple[str, Path]] = []

    for testroot in projects:
        source_dir = rootdir / f"test-{testroot}"
        expect_dir = source_dir / "expect"
        for sample_path in expect_dir.glob("**/*"):
            if sample_path.is_dir():
                continue
            expect_file = sample_path.relative_to(expect_dir)
            parameters.append((testroot, expect_file))

    return pytest.mark.parametrize(
        ("rootdir", "expect_file"),
        [
            pytest.param(
                path(rootdir),
                expect_file,
                marks=[pytest.mark.sphinx("mdx", testroot=testroot)],
                id=f"{root.name}/{testroot}/{expect_file}",
            )
            for (testroot, expect_file) in parameters
        ],
        # these have to be picked up by the Sphinx fixture
        indirect=("rootdir",),
    )
