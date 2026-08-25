import asyncio
import json
from ._brimp import Browser as _Browser, CancellationToken as _CancellationToken

class BrimpError(RuntimeError):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code

class BrimpCancelledError(asyncio.CancelledError):
    code = "cancelled"

def _translate(error: RuntimeError) -> BrimpError:
    message = str(error)
    if message.startswith("brimp ") and ": " in message:
        code, detail = message.removeprefix("brimp ").split(": ", 1)
        return BrimpError(code, detail)
    return BrimpError("internal", message)

async def _native(call, *args):
    try:
        return await asyncio.to_thread(call, *args)
    except RuntimeError as error:
        raise _translate(error) from error

class Page:
    def __init__(self, inner): self._inner = inner
    async def goto(self, url: str, *, timeout: float = 30.0):
        token = _CancellationToken()
        future = asyncio.get_running_loop().run_in_executor(None, self._inner.goto, url, int(timeout * 1000), token)
        try:
            return await asyncio.shield(future)
        except asyncio.CancelledError:
            token.cancel()
            try: await asyncio.shield(future)
            except Exception: pass
            raise BrimpCancelledError() from None
        except RuntimeError as error:
            raise _translate(error) from error
    async def evaluate(self, expression: str):
        return json.loads(await _native(self._inner.evaluate, expression))
    async def title(self): return await _native(self._inner.title)
    async def text_content(self): return await _native(self._inner.text_content)
    async def screenshot(self, *, full_page: bool = False): return await _native(self._inner.screenshot, full_page)
    async def close(self): return await _native(self._inner.close)

class Browser:
    def __init__(self, inner): self._inner = inner
    async def new_page(self): return Page(await _native(self._inner.new_page))
    async def close(self): return await _native(self._inner.close)

async def launch(*, persona_json: str | None = None):
    return Browser(await _native(_Browser.launch, persona_json))

__all__ = ["BrimpCancelledError", "BrimpError", "Browser", "Page", "launch"]
