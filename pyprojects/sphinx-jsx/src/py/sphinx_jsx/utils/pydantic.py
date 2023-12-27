from typing import Any, Dict, Set, Tuple, Type, TypeVar, Union, get_args, get_origin

from pydantic import BaseModel
from typing_extensions import TypeGuard

T = TypeVar("T")


def update_forward_refs(global_ns: Dict):
    models: Set[Type[BaseModel]] = set()

    def collect_models(**items: Any):
        for v in items.values():
            try:
                if v in models:
                    continue
            except TypeError:
                continue
            try:
                is_model = issubclass(v, BaseModel)
            except TypeError:
                continue
            if is_model:
                models.add(v)
            try:
                collect_models(**vars(v))
            except TypeError:
                continue

    collect_models(**global_ns)

    for v in models:
        v.update_forward_refs()


def is_instance_of_type(obj: Any, annotation: Type[T]) -> TypeGuard[T]:
    def extract_types(annotation) -> Tuple:
        origin = annotation
        while True:
            args = get_args(origin)
            origin = get_origin(origin)
            if origin is Union:
                return tuple(t for subtype in args for t in extract_types(subtype))
            if origin is None:
                return args or (annotation,)

    types = extract_types(annotation)

    if any(t is Any for t in types):
        return True

    return isinstance(obj, types)
