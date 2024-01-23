from pydantic import BaseModel


class JSXSymbols(BaseModel):
    jsx: str = "_jsx"
    jsxs: str = "_jsxs"
    fragment: str = "_Fragment"
    trans: str = "_Trans"
    gettext: str = "_gettext"
    url: str = "_url"
