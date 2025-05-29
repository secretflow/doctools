project = "Test project"

extensions = [
    "myst_nb",
    "secretflow_doctools",
    "sphinx.ext.autodoc",
    "sphinx.ext.graphviz",
    "sphinxcontrib.mermaid",
]

suppress_warnings = ["myst.strikethrough", "ref.term"]

language = "en"
locale_dirs = ["./locales/"]
gettext_compact = False
gettext_uuid = False
gettext_allow_fuzzy_translations = True

exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

myst_enable_extensions = [
    "amsmath",
    "dollarmath",
    "strikethrough",
]
myst_gfm_only = False
myst_heading_anchors = 6

nb_execution_mode = "auto"

nb_mime_priority_overrides = [
    ("mdx", "text/html", 10),
    ("mdx", "image/svg+xml", 20),
    ("mdx", "image/png", 21),
    ("mdx", "image/jpeg", 22),
    ("mdx", "image/gif", 23),
    ("mdx", "text/markdown", 30),
    ("mdx", "text/plain", 31),
    ("mdx", "text/latex", 32),
    ("mdx", "application/javascript", None),
    ("mdx", "application/vnd.jupyter.widget-view+json", None),
    ("mdx", "application/vnd.code.notebook.error", None),
]
