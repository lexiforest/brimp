# Exceptions

Catch `brimp.BrimpError` for Brimp failures, or `brimp.RequestException` for HTTP
request failures. Every exception below is available from both `brimp` and
`brimp.exceptions`. `brimp.RequestsError` is another spelling of
`brimp.RequestException`. There is no `CurlError` base or public low-level Curl
object.

Request exceptions expose `.code`, the numeric curl error code, or `0` when the
failure comes from Brimp's validation, redirect policy or operation deadline.
Classification also uses native CONNECT and TLS verification results. For
example, a certificate failure can raise `CertificateVerifyError` while `.code`
retains a generic connection code returned after HTTP/3 fallback.

They also expose `.response` and `.page`. Both refer to the Page/AsyncPage for
`raise_for_status()` and `json()` failures. Native transfer failures currently
leave these fields as `None`, including failures after partial response data has
arrived; a previous response on a reused page is never attached to that failure.

```python
import brimp

try:
    with brimp.get("https://example.com/") as page:
        page.raise_for_status()
except brimp.CertificateVerifyError as error:
    print("Certificate verification failed", error.code)
except brimp.RequestException as error:
    print("Request failed", error.code)
```

HTTP status codes alone do not raise. Call `page.raise_for_status()` to raise
`brimp.HTTPError` for a 4xx/5xx response. Its `.status_code` and `.url` retain the
failed response's values even if the attached live page navigates again.
`HTTPError` also represents curl HTTP protocol failures, for which these fields
are `None` when there is no response object.

`RequestTimeout` inherits both `TransportError` and `Timeout`. A deadline reached
while fetching the navigation response or reading a stream raises
`RequestTimeout`. A deadline reached while executing page JavaScript or waiting
for browser operations raises `Timeout`, which is not a `RequestException`.
Connect and read phases are not yet exposed as separate exception types.

Runtime errors such as `JavaScriptError`, `Reentry` and `StaleHandle` remain
separate from HTTP failures. Request exceptions also inherit `RuntimeError` and
`OSError`. URL/header validation errors inherit `InvalidInput` and `ValueError`;
`SessionClosed` inherits `Closed`. Async operations use the same exception types,
and task cancellation still raises `asyncio.CancelledError`.

`JSONDecodeError` also inherits the standard library's `json.JSONDecodeError`,
retains its decoder position fields, and chains the original decoding error.

## Common bases and runtime errors

::: brimp.BrimpError

::: brimp.RuntimeError

::: brimp.RequestException

::: brimp.TransportError

::: brimp.InvalidInput

::: brimp.JavaScriptError

::: brimp.Timeout

::: brimp.Closed

::: brimp.Reentry

::: brimp.StaleHandle

## HTTP and transport errors

::: brimp.ConnectionError

::: brimp.DNSError

::: brimp.ProxyError

::: brimp.SSLError

::: brimp.CertificateVerifyError

::: brimp.RequestTimeout

::: brimp.HTTPError

::: brimp.IncompleteRead

::: brimp.TooManyRedirects

::: brimp.SessionClosed

::: brimp.ImpersonateError

::: brimp.InterfaceError

## Validation and decoding errors

::: brimp.InvalidURL

::: brimp.MissingSchema

::: brimp.InvalidSchema

::: brimp.InvalidProxyURL

::: brimp.InvalidHeader

::: brimp.InvalidJSONError

::: brimp.JSONDecodeError
