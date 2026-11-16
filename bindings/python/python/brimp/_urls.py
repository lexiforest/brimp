"""URL construction shared by Python navigation and streaming requests."""

import json
import re
from collections import Counter
from urllib.parse import parse_qsl, quote as percent_encode, urlencode, urljoin, urlparse


from .exceptions import InvalidURL, InvalidSchema, MissingSchema


_SAFE = "!#$%&'()*+,/:;=?@[]~"
_UNRESERVED = frozenset("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~")


def validate_base_url(base_url):
    if base_url:
        try:
            parsed = urlparse(base_url)
        except ValueError as error:
            raise InvalidURL(str(error)) from error
        if not (parsed.scheme and parsed.hostname):
            raise InvalidURL("base_url must be an absolute URL")


def _merge_params(url, params):
    parsed = urlparse(url)
    existing = parse_qsl(parsed.query, keep_blank_values=True)
    incoming = list(params.items()) if hasattr(params, "items") else list(params)
    old_counts = Counter(key for key, _ in existing)
    new_counts = Counter(key for key, _ in incoming)
    for key, value in incoming:
        if isinstance(value, (bool, dict)):
            value = json.dumps(value)
        if old_counts[key] == new_counts[key] == 1:
            existing = [(name, value if name == key else item) for name, item in existing]
        else:
            existing.append((key, value))
    return parsed._replace(query=urlencode(existing, doseq=True)).geturl()


def _requote(url):
    def decode_unreserved(match):
        escape = match.group(1)
        if len(escape) != 2 or not escape.isalnum():
            return match.group()
        character = chr(int(escape, 16))
        return character if character in _UNRESERVED else match.group()

    try:
        normalized = re.sub(r"%([^%]{0,2})", decode_unreserved, url)
    except ValueError:
        return percent_encode(url, safe=_SAFE.replace("%", "") + "|")
    return percent_encode(normalized, safe=_SAFE + "|")


def build_url(url, *, base_url=None, default_params=None, params=None, quote=""):
    """Merge queries before joining the base, following curl-cffi's ordering.

    A unique incoming key replaces a unique existing key; repeated keys append.
    quote=False skips automatic quoting, but params still use form encoding.
    """
    validate_base_url(base_url)
    for values in (default_params, params):
        if values:
            url = _merge_params(url, values)
    if base_url:
        url = urljoin(base_url, url)
    if quote is not False and quote is not None and not isinstance(quote, str):
        raise TypeError("quote must be a string, False, or None")
    if quote:
        safe = "".join(character for character in _SAFE if character not in quote)
        parsed = urlparse(url)
        url = parsed._replace(
            path=percent_encode(parsed.path, safe=safe),
            query=urlencode(parse_qsl(parsed.query, keep_blank_values=True), doseq=True, safe=safe),
        ).geturl()
    return url if quote is False else _requote(url)


def validate_request_url(url):
    """Validate after base-URL resolution without changing native URL spelling."""
    try:
        parsed = urlparse(url)
        if not parsed.scheme:
            raise MissingSchema("request URL must include a scheme or use base_url")
        if parsed.scheme not in {"http", "https", "data", "about"}:
            raise InvalidSchema(f"unsupported request URL scheme: {parsed.scheme}")
        if parsed.scheme in {"http", "https"}:
            if not parsed.hostname:
                raise InvalidURL("request URL must include a hostname")
            # urllib validates ports lazily.
            _ = parsed.port
    except InvalidURL:
        raise
    except ValueError as error:
        raise InvalidURL(str(error)) from error
    return url
