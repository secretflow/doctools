from __future__ import annotations

from typing import List, Literal, Set, Union
from uuid import uuid4

from docutils import nodes
from loguru import logger
from pydantic import BaseModel, Field
from sphinx import addnodes
from sphinx.environment import BuildEnvironment
from sphinx.util.nodes import clean_astext

from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.pathfinding import Pathfinder

# after https://docusaurus.io/docs/sidebar/items


class SidebarItem(BaseModel):
    kind: Literal["doc", "link", "category"]
    key: str
    title: str
    children: List[SidebarItem] = Field(default_factory=list)


Sidebar = List[SidebarItem]


def generate_sidebar(
    root_doctree: nodes.document,
    pathfinder: Pathfinder,
    env: BuildEnvironment,
) -> Sidebar:
    seen: Set[str] = set()

    def resolve_doctree(doctree: nodes.document) -> Sidebar:
        root: Sidebar = []
        # current level of sidebar
        # either root or a category in case the sidebar has a caption
        curr: Sidebar = root

        for toctree in doctree.findall(addnodes.toctree):
            # will not honor :hidden: because its intended use was to hide a toctree
            # from the rendered page yet still include it in the master toctree
            # so that Sphinx will explicitly consider it a part of the documentation
            # which we should interpret as an intention to include it in the sidebar
            #
            # currently the only way to completely omit documentation from the sidebar
            # is to skip the toctree directive entirely
            # (which causes Sphinx to emit warnings)

            if toctree.get("caption"):
                category = SidebarItem(
                    kind="category",
                    key=random_id(),
                    title=toctree["caption"],
                )

                curr.append(category)
                curr = category.children

            for title, ref in toctree["entries"]:
                title: Union[str, None]
                ref: str

                if pathfinder.is_external_url(ref):
                    # external link
                    entry = SidebarItem(
                        kind="link",
                        key=ref,
                        title=title or ref,
                    )
                    curr.append(entry)
                    continue

                if ref == "self":
                    # 'self' refers to the document from which this toctree originates
                    # which we won't support
                    continue

                file = pathfinder.get_output_path(ref)
                file = file.relative_to(pathfinder.output_root)
                title = title or clean_astext(env.titles[ref])

                entry = SidebarItem(kind="doc", key=str(file), title=title)

                if ref in seen:
                    logger.warning(
                        _("ignoring circular sidebar ref {ref} -> {ref}"),
                        ref=ref,
                    )
                    continue

                seen.add(ref)

                items = resolve_doctree(env.get_doctree(ref))

                if items:
                    entry = SidebarItem(
                        kind="doc",
                        key=entry.key,
                        title=entry.title,
                        children=items,
                    )

                curr.append(entry)

            curr = root

        return root

    sitemap = resolve_doctree(root_doctree)
    return sitemap


def random_id():
    return f"\x00{uuid4()}"
