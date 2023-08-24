from __future__ import annotations

from typing import List, Union

from docutils import nodes
from pydantic import BaseModel
from sphinx import addnodes
from sphinx.environment import BuildEnvironment
from sphinx.util.nodes import clean_astext

from .pathfinding import Pathfinder


class ContentEntry(BaseModel):
    filepath: Union[str, None]
    external: Union[str, None] = None
    title: str
    hidden: bool = False
    children: List[ContentEntry] = []


def resolve_sitemap(
    root_doctree: nodes.document,
    pathfinder: Pathfinder,
    env: BuildEnvironment,
) -> List[ContentEntry]:
    def resolve_doctree(doctree: nodes.document) -> List[ContentEntry]:
        root: List[ContentEntry] = []

        for toctree in doctree.findall(addnodes.toctree):
            for title, ref in toctree["entries"]:
                title: Union[str, None]
                ref: str

                if pathfinder.is_external_url(ref):
                    # external link
                    entry = ContentEntry(
                        filepath=None,
                        external=ref,
                        title=title or ref,
                    )
                    root.append(entry)
                    continue

                if ref == "self":
                    # 'self' refers to the document from which this toctree originates
                    # which we won't support
                    continue

                file = pathfinder.get_output_path(ref)
                file = file.relative_to(pathfinder.output_root)
                title = title or clean_astext(env.titles[ref])

                entry = ContentEntry(filepath=str(file), title=title)
                root.append(entry)

                child = env.get_doctree(ref)
                entry.children = resolve_doctree(child)

        return root

    return resolve_doctree(root_doctree)
