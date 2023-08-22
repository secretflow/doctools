from contextlib import suppress
from pathlib import Path
from typing import Callable, Optional, Union

from loguru import logger
from twisted.internet import endpoints, reactor
from twisted.internet.error import CannotListenError
from twisted.python.failure import Failure
from twisted.web.server import Site
from twisted.web.static import File


def serve_directory_forever(
    root: Union[str, Path],
    port: int,
    before_start: Optional[Callable] = None,
):
    factory = Site(File(root))
    endpoint = endpoints.TCP4ServerEndpoint(reactor, port)

    def on_listening(*args):
        if before_start:
            before_start()

        logger.debug(f"Listening on port {port}")

        with suppress(KeyboardInterrupt):
            reactor.run()

    def on_error(failure: Failure):
        if failure.check(SystemExit, KeyboardInterrupt):
            return
        if failure.check(CannotListenError):
            logger.error(f"Port {port} is already in use")
            return
        logger.error(failure.getTraceback())

    endpoint.listen(factory).addCallback(on_listening).addErrback(on_error)
