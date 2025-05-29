from collections import defaultdict
from typing import Any, Iterable


def into_messages(data: Any, keys: set[str]) -> dict[str, list[str]]:
    messages: dict[str, list[str]] = defaultdict(list)

    for k, v in iter_json_pointer(data, ()):
        if len(k) == 0:
            continue
        if isinstance(v, str) and k[-1] in keys:
            messages[v].append(into_json_pointer(k))

    return {**messages}


def iter_json_pointer(
    data: Any,
    path: tuple[str, ...],
) -> Iterable[tuple[tuple[str, ...], Any]]:
    """https://datatracker.ietf.org/doc/html/rfc6901"""

    yield path, data

    if isinstance(data, dict):
        for k, v in data.items():
            yield from iter_json_pointer(v, (*path, k))
    elif isinstance(data, (tuple, list)):
        for i, v in enumerate(data):
            yield from iter_json_pointer(v, (*path, str(i)))


def mutate_json(data: Any, updates: dict[str, Any]):
    lookup: dict[tuple[str, ...], dict[str, Any]] = defaultdict(dict)

    for k, v in updates.items():
        path = from_json_pointer(k)
        if len(path) == 0:
            raise ValueError("unsupported: update with an empty path")
        *prefix, name = path
        lookup[tuple(prefix)][name] = v

    def visit_mut(data: Any, prefix: tuple[str, ...]):
        updates = lookup[prefix]
        if isinstance(data, dict):
            for k in data.keys():
                if k in updates:
                    data[k] = updates[k]
                visit_mut(data[k], (*prefix, k))
        if isinstance(data, list):
            for i in range(len(data)):
                k = str(i)
                if k in updates:
                    data[i] = updates[k]
                visit_mut(data[i], (*prefix, k))

    visit_mut(data, ())


def into_json_pointer(segments: Iterable[str]) -> str:
    items = (s.replace("~", "~0").replace("/", "~1") for s in segments)
    items = (f"/{s}" for s in items)
    return "".join(items)


def from_json_pointer(pointer: str) -> tuple[str, ...]:
    items = pointer.split("/")[1:]
    items = (s.replace("~1", "/").replace("~0", "~") for s in items)
    return tuple(items)
