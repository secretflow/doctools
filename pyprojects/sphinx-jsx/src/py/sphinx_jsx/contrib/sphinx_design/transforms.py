from docutils import nodes
from sphinx.transforms.post_transforms import SphinxPostTransform

from sphinx_jsx.syntax import rst

from .elements import container__sphinx_design_card


class SphinxDesignPostTransform(SphinxPostTransform):
    default_priority = 100
    builders = ("jsx",)

    def run(self, **kwargs) -> None:
        for node in self.document.findall(nodes.container):
            component_type = node.get("design_component")
            if component_type == "card":
                rst.transforms.specialize_to(container__sphinx_design_card, node)
