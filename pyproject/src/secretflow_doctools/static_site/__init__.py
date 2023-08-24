"""Manage the lifecycle of a Sphinx scaffolding, such as installing node dependencies \
    starting the devserver, and building the site."""

from .__main__ import cli
from .runner import NodeProjectOptions, NodeProjectRunner

__all__ = ["cli", "NodeProjectRunner", "NodeProjectOptions"]
