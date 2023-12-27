from typing import Protocol


class RenderingContext(Protocol):
    def resolve_reference(self, ref: str):
        pass
