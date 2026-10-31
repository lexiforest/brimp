# Exceptions

These names are exported by `brimp`. HTTP error status codes alone do not raise;
call `page.raise_for_status()` to raise `brimp.HTTPError` for a 4xx/5xx response.
Transport and runtime failures raise during the operation itself.

::: brimp.HTTPError

::: brimp.RuntimeError

::: brimp.InvalidInput

::: brimp.JavaScriptError

::: brimp.Timeout

::: brimp.Closed

::: brimp.Reentry

::: brimp.StaleHandle

::: brimp.TransportError
