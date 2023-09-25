from __future__ import annotations

from typing import Any, Set

from docutils import nodes
from sphinx.transforms.post_transforms import SphinxPostTransform


class NbSphinxCellTransform(SphinxPostTransform):
    """Wrap adjacent nbinput, nboutput, and nblast nodes in a container, so that it is\
        easier to do tree traversal on them."""

    default_priority = 100

    def run(self, **kwargs: Any) -> None:
        if not self.app.builder.tags.has("mdx"):
            return

        def is_nb_cell(node: nodes.container):
            if not isinstance(node, nodes.container):
                return
            classes = node.get("classes", [])
            return "nbinput" in classes or "nboutput" in classes

        processed: Set[nodes.Node] = set()

        for node in self.document.findall(is_nb_cell):
            if node in processed:
                continue

            processed.add(node)

            container = nodes.container("")
            node.replace_self(container)

            container["classes"] = ["nbcell"]
            container.append(node)

            if "nblast" in node.get("classes", []):
                continue

            for next_node in container.findall(
                nodes.container,
                include_self=False,
                descend=False,
                siblings=True,
            ):
                next_classes = next_node.get("classes", [])
                if "nbinput" in next_classes or "nboutput" in next_classes:
                    next_node.parent.remove(next_node)
                    container.append(next_node)
                    processed.add(next_node)
                if "nblast" in next_node.get("classes", []):
                    break
