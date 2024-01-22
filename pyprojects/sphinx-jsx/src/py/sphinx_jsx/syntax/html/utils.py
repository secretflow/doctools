from bs4 import BeautifulSoup
from docutils.utils import new_document

# from sphinx_jsx._lib import mdx_to_js
# from sphinx_jsx.translator.intrinsic import IntrinsicMarkupTranslator
from .elements import html_element
from .transforms import consume_soup


def html_to_jsx(source: str) -> str:
    soup = BeautifulSoup(source, "lxml")
    tree = html_element("", name="div")
    consume_soup({}, tree, soup)
    document = new_document("<string>")
    document.append(tree)
    # translator = IntrinsicMarkupTranslator(document)
    # document.walkabout(translator)
    # return translator.render()
    # return mdx_to_js(translator.render())
