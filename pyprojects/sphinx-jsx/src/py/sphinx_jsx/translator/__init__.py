from sphinx.util.docutils import SphinxTranslator

from .base import BaseJSXTranslator
from .containers import ContainerMarkupTranslator
from .figures import FigureMarkupTranslator
from .ignored import IgnoredMarkupTranslator
from .intrinsic import IntrinsicMarkupTranslator
from .lists import ListMarkupTranslator
from .math import MathMarkupTranslator
from .references import ReferenceMarkupTranslator
from .special import SpecialMarkupTranslator
from .symbol import SymbolMarkupTranslator
from .tables import TableMarkupTranslator


class DocutilsJSXTranslator(
    ContainerMarkupTranslator,
    FigureMarkupTranslator,
    IgnoredMarkupTranslator,
    IntrinsicMarkupTranslator,
    ListMarkupTranslator,
    MathMarkupTranslator,
    ReferenceMarkupTranslator,
    SpecialMarkupTranslator,
    SymbolMarkupTranslator,
    TableMarkupTranslator,
    BaseJSXTranslator,
):
    pass


class SphinxJSXTranslator(DocutilsJSXTranslator, SphinxTranslator):
    pass
