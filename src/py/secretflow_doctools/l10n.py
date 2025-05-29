from gettext import NullTranslations, translation
from importlib.resources import files
from typing import Optional

catalog: Optional[NullTranslations] = None


def gettext(msg: str) -> str:
    global catalog

    if catalog is None:
        import secretflow_doctools

        locales = str(files(secretflow_doctools).joinpath("locales"))
        catalog = translation("messages", locales, fallback=True)

    return catalog.gettext(msg)
