import re
from typing import List

from docutils import nodes

from sphinx_jsx.translator import BaseJSXTranslator
from sphinx_jsx.translator.special import Special

from . import elements, tags

# [42]:
RE_CELL_PROMPT = re.compile(r".*\[(\d+)\]:\s*")


def visit_container(self: BaseJSXTranslator, node: elements.container__nbsphinx):
    self.enter_nesting(node, tags.NotebookCell())


def visit_code_area(self: BaseJSXTranslator, node: nodes.Element):
    self.enter_nesting(
        node,
        tags.NotebookCell._CodeArea(
            prompt=node.get("prompt"),
            stderr=node.get("stderr", False),
        ),
    )


def depart_code_area(self: BaseJSXTranslator, node: nodes.Element):
    code_area = self.leave_nesting(node, tags.NotebookCell._CodeArea)

    if "nbinput" in code_area.classnames:
        cell = tags.NotebookCell.Source()
    elif code_area.stderr:
        cell = tags.NotebookCell.StdErr()
    elif not code_area.prompt:
        cell = tags.NotebookCell.StdOut()
    else:
        cell = tags.NotebookCell.Output()

    cell.prompt = code_area.prompt or ""
    cell.children.extend(code_area.children)

    for raw in code_area.filter_children(Special.Raw):
        cell.fallback[raw.format] = raw.content

    self.append_child(node, cell)


def depart_container(self: BaseJSXTranslator, node: elements.container__nbsphinx):
    container = self.leave_nesting(node, tags.NotebookCell)

    container.remove_children(tags.NotebookCell._CodeArea)

    prompts: List[str] = []

    for source in container.remove_children(tags.NotebookCell.Source):
        container.source = source
        prompts.append(source.prompt)
        break

    for output in container.remove_children(tags.NotebookCell.Output):
        container.output = output
        prompts.append(output.prompt)
        break

    for stdout in container.remove_children(tags.NotebookCell.StdOut):
        container.stdout = stdout
        prompts.append(stdout.prompt)
        break

    for stderr in container.remove_children(tags.NotebookCell.StdErr):
        container.stderr = stderr
        prompts.append(stderr.prompt)
        break

    for prompt in prompts:
        matched = RE_CELL_PROMPT.match(prompt)
        if not matched:
            continue
        try:
            container.number = int(matched.group(1))
        except ValueError:
            continue
