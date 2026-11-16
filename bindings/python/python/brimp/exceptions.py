"""Request failures and browser-runtime failures share BrimpError.

Request exceptions carry a numeric curl code (0 for non-curl failures) and an
optional response. Runtime timeouts and HTTP request timeouts remain distinct.
"""

import json as _json


class BrimpError(Exception):
    """Base class for Brimp failures."""


class RuntimeError(BrimpError):
    """A browser-runtime or host operation failed."""


class InvalidInput(RuntimeError):
    """Invalid arguments or unsupported input configuration."""


class JavaScriptError(RuntimeError):
    """Page JavaScript evaluation failed."""


class Timeout(RuntimeError):
    """A runtime operation or HTTP request exceeded its deadline."""


class Closed(RuntimeError):
    """The page, session or handle has been closed."""


class Reentry(RuntimeError):
    """A host callback attempted to reenter its page or session."""


class StaleHandle(RuntimeError):
    """A handle belongs to an expired realm or was disposed."""


class RequestException(RuntimeError, OSError):
    """Base for HTTP request failures; response is a Page/AsyncPage or None."""

    def __init__(self, message, code=0, response=None):
        super().__init__(message)
        self.code = int(code)
        self.response = response
        self.page = response


# curl_cffi's requests API exposes both spellings for the same exception.
RequestsError = RequestException


class TransportError(RequestException):
    """A network transfer failed."""


class ConnectionError(TransportError):
    """Connection establishment or communication failed."""


class DNSError(ConnectionError):
    """The destination hostname could not be resolved."""


class ProxyError(TransportError):
    """Proxy resolution, negotiation or tunneling failed."""


class SSLError(ConnectionError):
    """TLS negotiation or configuration failed."""


class CertificateVerifyError(SSLError):
    """The peer certificate could not be verified."""


class RequestTimeout(TransportError, Timeout):
    """An HTTP transfer exceeded its deadline, unlike a JavaScript timeout."""


class HTTPError(TransportError):
    """HTTP status or protocol failure; status_code and url snapshot the response."""

    def __init__(self, message, code=0, response=None):
        super().__init__(message, code, response)
        self.status_code = None if response is None else response.status_code
        self.url = None if response is None else response.url


class IncompleteRead(HTTPError):
    """The response ended before all declared bytes arrived."""


class TooManyRedirects(TransportError):
    """The redirect limit was exceeded."""


class SessionClosed(RequestException, Closed):
    """An operation used a closed session."""


class ImpersonateError(RequestException, InvalidInput):
    """The transport impersonation profile could not be applied."""


class InvalidURL(RequestException, InvalidInput, ValueError):
    """The request URL is malformed."""


class MissingSchema(InvalidURL):
    """The request URL has no scheme and no base URL resolves it."""


class InvalidSchema(InvalidURL):
    """The request URL uses an unsupported scheme."""


class InvalidProxyURL(InvalidURL):
    """The proxy URL is malformed or unsupported."""


class InvalidHeader(RequestException, InvalidInput, ValueError):
    """A request header name or value is invalid."""


class InterfaceError(TransportError):
    """The configured outgoing interface could not be used."""


class InvalidJSONError(RequestException):
    """The response body could not be decoded as JSON."""


class JSONDecodeError(InvalidJSONError, _json.JSONDecodeError):
    """JSON failure retaining the stdlib decoder's msg, doc and pos fields."""

    def __init__(self, message, doc, pos, *, response=None):
        _json.JSONDecodeError.__init__(self, message, doc, pos)
        RequestException.__init__(self, self.args[0], response=response)

    def __reduce__(self):
        return type(self), (self.msg, self.doc, self.pos), self.__dict__
