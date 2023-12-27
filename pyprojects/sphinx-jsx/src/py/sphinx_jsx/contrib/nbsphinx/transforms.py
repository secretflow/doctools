from typing import List, cast

from docutils import nodes
from sphinx.transforms.post_transforms import SphinxPostTransform
from typing_extensions import TypeGuard

from sphinx_jsx.syntax import rst

from .elements import container__nbsphinx


class NbSphinxCellTransform(SphinxPostTransform):
    default_priority = 901
    builders = ("jsx",)

    def run(self, **kwargs) -> None:
        from nbsphinx import CodeAreaNode, FancyOutputNode

        def is_unprocessed_cell(node: nodes.Node) -> TypeGuard[nodes.container]:
            if isinstance(node, container__nbsphinx):
                return False
            if not isinstance(node, nodes.container):
                return False
            classes = node.get("classes", [])
            return bool("nbinput" in classes or "nboutput" in classes)

        def is_code_area(node: nodes.Node):
            return isinstance(node, (CodeAreaNode, FancyOutputNode))

        for node in self.document.findall(is_unprocessed_cell):
            node: nodes.container
            parent = node.parent

            head = container__nbsphinx("")
            parent.insert(parent.index(node), head)

            containers: List[nodes.container] = []

            for container in node.findall(
                is_unprocessed_cell,
                include_self=True,
                descend=False,
                siblings=True,
            ):
                containers.append(container)
                if "nblast" in container.get("classes", []):
                    break

            for container in containers:
                classes = container.get("classes", [])

                code_area = cast(nodes.Element, container.next_node(is_code_area))

                if not code_area:
                    continue

                code_area["classes"] = classes

                for inner in code_area.findall(nodes.container):
                    rst.transforms.unwrap(inner)

                rst.transforms.move_to(head, code_area)
                rst.transforms.pull(container)
