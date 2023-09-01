from typing import Optional, Union

from sphinx.util import logging


def get_logger(name: str):
    logger = logging.getLogger(name)

    _logging_fn = logger.log

    def log(
        level: Union[int, str],
        msg: str,
        *args,
        subtype: Optional[str] = None,
        **kwargs,
    ):
        return _logging_fn(
            level,
            f"{msg} [mdx.%(subtype)s]",
            *args,
            subtype=subtype or "warning",
            **kwargs,
        )

    logger.log = log

    return logger
