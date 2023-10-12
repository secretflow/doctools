from __future__ import annotations

from typing import List, Literal, Union

from docutils import nodes
from pydantic import BaseModel
from sphinx import addnodes
from sphinx.environment import BuildEnvironment
from sphinx.util.nodes import clean_astext

from .pathfinding import Pathfinder
from .utils.logging import get_logger

logger = get_logger(__name__)

# after https://docusaurus.io/docs/sidebar/items


class SidebarItemDoc(BaseModel):
    type: Literal["doc"] = "doc"
    id: str  # this is currently the file path
    label: str


class SidebarItemLink(BaseModel):
    type: Literal["link"] = "link"
    href: str
    label: str


class SidebarItemCategory(BaseModel):
    type: Literal["category"] = "category"
    label: str
    items: List[Union[SidebarItemDoc, SidebarItemLink, SidebarItemCategory]]
    link: Union[SidebarItemDoc, None] = None


SidebarItem = Union[SidebarItemDoc, SidebarItemLink, SidebarItemCategory]
Sidebar = List[SidebarItem]


def generate_sidebar(
    root_doctree: nodes.document,
    pathfinder: Pathfinder,
    env: BuildEnvironment,
) -> Sidebar:
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
                category = SidebarItemCategory(label=toctree["caption"], items=[])
                curr.append(category)
                curr = category.items

            for title, ref in toctree["entries"]:
                title: Union[str, None]
                ref: str

                if pathfinder.is_external_url(ref):
                    # external link
                    entry = SidebarItemLink(label=title or ref, href=ref)
                    curr.append(entry)
                    continue

                if ref == "self":
                    # 'self' refers to the document from which this toctree originates
                    # which we won't support
                    continue

                file = pathfinder.get_output_path(ref)
                file = file.relative_to(pathfinder.output_root)
                title = title or clean_astext(env.titles[ref])

                entry = SidebarItemDoc(label=title, id=str(file))

                items = resolve_doctree(env.get_doctree(ref))
                if items:
                    entry = SidebarItemCategory(label=title, items=items, link=entry)

                curr.append(entry)

            curr = root

        return root

    sitemap = resolve_doctree(root_doctree)
    return sitemap
