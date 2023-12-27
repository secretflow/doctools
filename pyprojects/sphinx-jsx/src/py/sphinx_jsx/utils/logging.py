import asyncio
import inspect
import logging
import os
import sys
from pathlib import Path
from types import ModuleType
from typing import Any, Optional, Tuple, Union

import loguru
from docutils.nodes import Node

IGNORED_ERRORS = (asyncio.TimeoutError,)


def qualname_tuple(obj: Any) -> Tuple[Optional[str], Optional[str]]:
    module_name = getattr(inspect.getmodule(obj), "__name__", None)
    obj_name = (
        getattr(obj, "__qualname__", None)
        or getattr(obj, "__name__", None)
        or getattr(obj, "co_name", None)
        or getattr(obj, "name", None)
    )
    return module_name, obj_name


def qualname(obj: Any) -> str:
    if isinstance(obj, ModuleType):
        return obj.__name__
    module_name, obj_name = qualname_tuple(obj)
    return f"{module_name or '<unknown_module>'}.{obj_name or '<unknown>'}"


class InterceptHandler(logging.Handler):
    def emit(self, record):
        # Get corresponding Loguru level if it exists
        try:
            level = loguru.logger.level(record.levelname).name
        except ValueError:
            level = record.levelno

        # Try to find the caller that generated the logged message
        frame, depth = logging.currentframe().f_back, 2
        while frame and frame.f_code.co_filename == logging.__file__:
            frame = frame.f_back
            depth += 1

        loguru.logger.opt(
            depth=depth,
            exception=record.exc_info,
        ).log(level, record.getMessage())


def no_spinner():
    return os.environ.get("CI") or not sys.stderr.isatty()


def _format_rst_node(origin: Node) -> str:
    file = origin.source or "<unknown_document>"
    file = file.replace("<", "\\<")
    line = origin.line or "?"
    return f"{file}:{line}"


def formatter_tty(record: "loguru.Record") -> str:
    if record["extra"].get("raw_output"):
        return "{message}\n"

    prefix = "<bold><level>{level: <8}</level></bold>"
    message = "{message}"
    if record["level"].no >= logging.WARNING:
        message = "<level>{message}</level>"
    if record["exception"] and record["exception"].type not in IGNORED_ERRORS:
        fmt = f"{prefix} {message}\n{{exception}}"
    else:
        fmt = f"{prefix} {message} <dim>[{{name}}:{{function}}:{{line}}]</dim>"

    origin = record["extra"].get("origin")

    if isinstance(origin, Node):
        fmt = f"{fmt} <dim>[{_format_rst_node(origin)}]</dim>"

    return f"{fmt}\n"


def formatter_ci(record: "loguru.Record") -> str:
    if record["extra"].get("raw_output"):
        return "{message}\n"

    fmt = (
        "{time:YYYY-MM-DD HH:mm:ss.SSS ZZ} {level: <8}"
        " {message} [{name}:{function}:{line}]"
    )

    origin = record["extra"].get("origin")

    if isinstance(origin, Node):
        fmt = f"{fmt} [{_format_rst_node(origin)}]"

    if record["exception"] and record["exception"].type not in IGNORED_ERRORS:
        fmt = f"{fmt}\n{{exception}}"

    return f"{fmt}\n"


def configure_logging(
    *,
    log_file: Optional[Union[str, Path]] = None,
    level: Union[int, str] = logging.INFO,
):
    if log_file or no_spinner():
        formatter = formatter_ci
    else:
        formatter = formatter_tty
    loguru.logger.configure(
        handlers=[
            {
                "sink": log_file or sys.stderr,
                "level": level,
                "format": formatter,
            },
        ],
        levels=[
            {"name": "DEBUG", "color": "<magenta>"},
            {"name": "INFO", "color": "<blue>"},
            {"name": "SUCCESS", "color": "<bold><green>"},
            {"name": "WARNING", "color": "<yellow>"},
            {"name": "ERROR", "color": "<red>"},
            {"name": "CRITICAL", "color": "<bold><red>"},
        ],
    )
    logging.basicConfig(handlers=[InterceptHandler()], level=0, force=True)
