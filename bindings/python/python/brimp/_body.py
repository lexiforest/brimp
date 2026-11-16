"""Buffered HTTP body encoding shared by page requests and response streams."""

import json as _json
from collections.abc import Mapping
from urllib.parse import urlencode

from .exceptions import InvalidInput


def encode_body(*, body, data, content, json, headers, default_headers):
    """Return bytes or None and add a content type only when none is configured."""
    if body is not None and any(value is not None for value in (data, content, json)):
        raise InvalidInput("specify body or data/content/json, not both")

    content_type = None
    if json is not None:
        encoded = _json.dumps(json, separators=(",", ":")).encode()
        content_type = "application/json"
    elif content is not None:
        encoded = _raw_body(content, "content")
        content_type = "application/octet-stream"
    elif data is not None:
        if isinstance(data, (Mapping, list, tuple)):
            encoded = urlencode(data).encode()
            content_type = "application/x-www-form-urlencoded"
        else:
            encoded = _raw_body(data, "data")
            content_type = "application/octet-stream"
    elif body is not None:
        encoded = _raw_body(body, "body")
    else:
        return None

    if content_type is not None and not any(
        name.lower() == "content-type" for name, _ in (*default_headers, *headers)
    ):
        headers.append(("Content-Type", content_type))
    return encoded


def _raw_body(value, argument):
    if isinstance(value, str):
        return value.encode()
    if isinstance(value, (bytes, bytearray)):
        return bytes(value)
    raise TypeError(
        f"{argument} must be a string or bytes"
        + (", or a form mapping/sequence of pairs" if argument == "data" else "")
        + "; file-like and iterable uploads are unsupported"
    )
