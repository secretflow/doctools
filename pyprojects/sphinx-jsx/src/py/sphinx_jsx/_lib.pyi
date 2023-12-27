from typing import Literal, Optional

def mdx_to_js(
    source: str,
    *,
    development: Optional[bool] = None,
    mdx_provider: Optional[str] = None,
    jsx_runtime: Optional[str] = None,
) -> str: ...
def math_to_html(
    source: str,
    *,
    mode: Literal["inline", "block"] = "inline",
) -> str: ...
