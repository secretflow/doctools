from collections import defaultdict
from collections.abc import Iterable
from io import StringIO
from typing import Literal, Optional

import thefuzz.process
from docutils import nodes
from docutils.nodes import document
from pydantic import BaseModel, RootModel
from ruamel.yaml import YAML
from ruamel.yaml.comments import CommentedMap, CommentedSeq
from ruamel.yaml.scalarstring import LiteralScalarString
from sphinx.application import Sphinx
from sphinx.builders import Builder
from sphinx.environment import BuildEnvironment
from sphinx.locale import __
from sphinx.util.display import progress_message
from sphinx.util.docutils import SphinxTranslator

from secretflow_doctools.i18n.json import into_messages, mutate_json
from secretflow_doctools.options import GettextConfig, parse_config
from secretflow_doctools.utils.logging import configure_logging


class SwaggerGettextBuilder(Builder):
    name = "gettext-swagger"
    versioning_method = "none"
    versioning_compare = False
    use_message_catalog = False
    file_extension = ".mdx"

    staging: dict[str, "Messages"] = {}
    options: GettextConfig

    def __init__(self, app: Sphinx, env: BuildEnvironment) -> None:
        super().__init__(app, env)
        self.options = parse_config(app.config, GettextConfig)
        configure_logging()

    def get_messages_path(self, docname: str):
        path = (
            self.app.srcdir.joinpath(self.options.locale_dirs[0])
            .joinpath(self.options.language)
            .joinpath("LC_MESSAGES")
            .joinpath(docname)
        )
        path = path.with_name(f"{path.name}.swagger.yaml")
        return path

    def load_messages(self, docname: str) -> Optional["Messages"]:
        try:
            with open(self.get_messages_path(docname), "r") as f:
                old = yaml.load(f)
                old = Messages.model_validate(old)
                return old
        except Exception:
            return None

    def write_doc(self, docname: str, doctree: document) -> None:
        visitor = SwaggerGettextTranslator(doctree, self)
        doctree.walkabout(visitor)
        if visitor.output:
            self.staging[docname] = visitor.output

    @progress_message(__("dumping object inventory"))
    def merge_messages(self):
        for docname, new in self.staging.items():
            old = self.load_messages(docname)
            if old:
                old = new.update(old)
            with StringIO() as f:
                f.write(new.to_yaml(self.app.config.language))
                if old:
                    f.write("\n\n")
                    f.write(old.to_comment(self.app.config.language))
                f = f.getvalue()
            out = self.get_messages_path(docname)
            out.parent.mkdir(parents=True, exist_ok=True)
            with open(out, "wb+") as g:
                g.write(f.encode("utf-8"))

    def finish(self) -> None:
        self.merge_messages()

    def get_outdated_docs(self) -> str | Iterable[str]:
        yield from self.env.found_docs

    def get_target_uri(self, docname: str, typ: str | None = None) -> str:
        return docname


class SwaggerGettextTranslator(SphinxTranslator):
    def __init__(self, document: document, builder: Builder) -> None:
        super().__init__(document, builder)
        self.count = 0
        self.messages = defaultdict[str, list[tuple[int, str]]](list)
        self.output: Optional[Messages] = None

    @classmethod
    def get_qualified_path(cls, count: int, path: str):
        return f"{count}#{path}"

    @classmethod
    def match_path(cls, qualified: str, id: int) -> Optional[str]:
        split = qualified.split("#", 1)
        if len(split) != 2:
            return None
        if split[0] != str(id):
            return None
        return split[1]

    def visit_literal_block(self, node: nodes.literal_block):
        if node.get("language") != "swagger":
            return
        schema = yaml.load(StringIO(node.astext()))
        for k, ps in into_messages(schema, {"summary", "title", "description"}).items():
            k = k.strip()
            self.messages[k].extend([(self.count, p) for p in ps])
        self.count += 1

    def depart_document(self, node: nodes.document):
        if not self.messages:
            return

        self.output = Messages([])

        for k, ps in self.messages.items():
            m = Message(
                original=k,
                translated=k,
                paths=[self.get_qualified_path(n, p) for (n, p) in ps],
            )
            self.output.root.append(m)

    def unknown_visit(self, node: nodes.Node) -> None:
        pass

    def unknown_departure(self, node: nodes.Node) -> None:
        pass


class Message(BaseModel):
    original: str
    translated: str
    paths: list[str]
    fuzzy: Optional[Literal[True]] = None


class Messages(RootModel):
    root: list[Message]

    def translate(self, id: int, data: str) -> str:
        data = yaml.load(StringIO(data))
        updates: dict[str, str] = {}
        for m in self.root:
            for p in m.paths:
                path = SwaggerGettextTranslator.match_path(p, id)
                if path:
                    updates[path] = LiteralScalarString(m.translated)
        mutate_json(data, updates)
        output = StringIO()
        yaml.dump(data, output)
        return output.getvalue()

    def update(self, that: "Messages") -> "Messages":
        old = {m.original: m for m in that.root}

        matched = set[str]()

        for new in self.root:
            exact = old.get(new.original)
            if exact is not None:
                new.translated = exact.translated
                new.fuzzy = exact.fuzzy
                matched.add(new.original)
                continue
            fuzzy = thefuzz.process.extractOne(
                new.original,
                old.keys(),
                score_cutoff=80,
            )
            if fuzzy is not None:
                key = fuzzy[0]
                new.translated = old[key].translated
                new.fuzzy = True
                matched.add(key)

        unmatched = Messages([])

        for old in that.root:
            if old.original not in matched:
                unmatched.root.append(old)

        return unmatched

    def to_yaml(self, lang: str):
        messages = []

        for m in self.root:
            d = [
                ("original", LiteralScalarString(m.original)),
                ("translated", LiteralScalarString(m.translated)),
                ("paths", m.paths),
            ]
            if m.fuzzy:
                d.insert(2, ("fuzzy", True))
            messages.append({k: v for k, v in d})

        messages = [CommentedMap(m) for m in messages]

        hint = {
            "zh_CN": "`original` 为原文，请修改 `translated` 为翻译, 语言={lang}",
        }.get(lang, "add translation here, language={lang}").format(lang=lang)

        for m in messages:
            m.yaml_set_comment_before_after_key("original", before=hint)
            if "fuzzy" in m:
                m.yaml_set_comment_before_after_key("fuzzy", before="\n")
            else:
                m.yaml_set_comment_before_after_key("paths", before="\n")

        messages = CommentedSeq(messages)

        for i in range(1, len(messages)):
            messages.yaml_set_comment_before_after_key(i, before="\n")

        output = StringIO()
        yaml.dump(messages, output)
        return output.getvalue()

    def to_comment(self, lang: str):
        return "\n".join([f"# {line}" for line in self.to_yaml(lang).split("\n")])

    def __bool__(self):
        return bool(self.root)


yaml = YAML(typ="rt")
yaml.default_flow_style = False
