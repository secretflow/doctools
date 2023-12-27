from __future__ import annotations

from inspect import _ParameterKind
from typing import List, Optional

from sphinx import addnodes

from sphinx_jsx.syntax.jsx.models import JSON_NULL, JSFragment, JSObject, JSXElement
from sphinx_jsx.utils.pydantic import update_forward_refs

from .base import BaseJSXTranslator


class PySymbol(JSObject):
    name: str
    uri: Optional[str] = None
    docstring: Optional[str] = None


class PyParameter(PySymbol):
    kind: _ParameterKind
    annotation: List[PySymbol] = []
    default: Optional[PySymbol] = None


class PySignature(PySymbol):
    modifier: Optional[PySymbol] = None
    module: Optional[PySymbol] = None
    params: List[PyParameter] = []
    returns: Optional[PySymbol] = None


class Symbol(JSXElement):
    signature: JSFragment = JSON_NULL
    content: JSFragment = JSON_NULL

    domain: str
    objtype: str

    symbol: Optional[PySymbol] = None

    class Signature(JSXElement):
        """See :py:class:`sphinx.addnodes.desc_signature`"""

        class Line(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_signature_line`"""

        class Note(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_annotation`"""

        class Prefix(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_addname`"""

        class Name(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_name`"""

        class ParameterList(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_parameterlist`"""

        class Parameter(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_parameter`"""

        class Keyword(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_keyword`"""

        class KeywordType(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_keyword_type`"""

        class ParameterName(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_name`"""

        class Punctuation(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_punctuation`"""

        class Operator(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_operator`"""

        class OptionalParameters(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_optional`"""

        class Space(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_sig_space`"""

        class Returns(JSXElement):
            """See :py:class:`sphinx.addnodes.desc_returns`"""

    class Content(JSXElement):
        """See :py:class:`sphinx.addnodes.desc_content`"""


class SymbolMarkupTranslator(BaseJSXTranslator):
    def visit_desc(self, node: addnodes.desc):
        self.enter_nesting(
            node,
            Symbol(domain=node["domain"], objtype=node["objtype"]),
        )

    def visit_desc_signature(self, node: addnodes.desc_signature):
        self.enter_nesting(node, Symbol.Signature())

    def visit_desc_annotation(self, node: addnodes.desc_annotation):
        self.enter_nesting(node, Symbol.Signature.Note())

    def visit_desc_sig_space(self, node: addnodes.desc_sig_space):
        self.enter_nesting(node, Symbol.Signature.Space())

    def visit_desc_addname(self, node: addnodes.desc_addname):
        self.enter_nesting(node, Symbol.Signature.Prefix())

    def visit_desc_name(self, node: addnodes.desc_name):
        self.enter_nesting(node, Symbol.Signature.Name())

    def visit_desc_parameterlist(self, node: addnodes.desc_parameterlist):
        self.enter_nesting(node, Symbol.Signature.ParameterList())

    def visit_desc_parameter(self, node: addnodes.desc_parameter):
        self.enter_nesting(node, Symbol.Signature.Parameter())

    def visit_desc_sig_name(self, node: addnodes.desc_sig_name):
        self.enter_nesting(node, Symbol.Signature.ParameterName())

    def visit_desc_sig_operator(self, node: addnodes.desc_sig_operator):
        self.enter_nesting(node, Symbol.Signature.Operator())

    def visit_desc_sig_punctuation(self, node: addnodes.desc_sig_punctuation):
        self.enter_nesting(node, Symbol.Signature.Punctuation())

    def visit_desc_signature_line(self, node: addnodes.desc_signature_line):
        self.enter_nesting(node, Symbol.Signature.Line())

    def visit_desc_sig_keyword(self, node: addnodes.desc_sig_keyword):
        self.enter_nesting(node, Symbol.Signature.Keyword())

    def visit_desc_sig_keyword_type(self, node: addnodes.desc_sig_keyword_type):
        self.enter_nesting(node, Symbol.Signature.KeywordType())

    def visit_desc_optional(self, node: addnodes.desc_optional):
        self.enter_nesting(node, Symbol.Signature.OptionalParameters())

    def visit_desc_returns(self, node: addnodes.desc_returns):
        self.enter_nesting(node, Symbol.Signature.Returns())

    def visit_desc_content(self, node: addnodes.desc_content):
        self.enter_nesting(node, Symbol.Content())

    def depart_desc(self, node: addnodes.desc):
        self.leave_nesting(node, Symbol)


update_forward_refs(globals())
