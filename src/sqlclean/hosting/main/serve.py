"""Answer one request from the sqlclean command on standard input and output."""

import json
import sys

from sqlclean.hosting._helpers.requests import handle_request


def serve() -> int:
    """Read one JSON request, write one JSON response and return the exit status."""

    request: dict[str, object] = json.loads(sys.stdin.read())
    response: dict[str, object] = handle_request(request)
    sys.stdout.write(json.dumps(response))
    return 0
