# pyv8c challenge examples, run against Brimp

`../pyv8c/examples` holds eight scripts that load a live anti-bot challenge
(SteamDB/Cloudflare "5s", Akamai, DataDome, F5 Shape, PX, hCaptcha, reCAPTCHA,
reese84) and print the cookie they obtained. They are the field test the original
question asks about: can Brimp do what they do?

## What can and cannot be answered here

- **The network works.** DNS answers in `198.18.0.0/15` because this machine uses
  a fake-IP TUN; real requests to the example targets succeed (`example.com`
  returns `200`, `steamdb.info` returns its interstitial). The default
  `http://127.0.0.1:7890` proxy of the examples is not needed and should be left
  off.
- **pyv8c itself does not run.** Its native `_pyv8` extension is not built in this
  checkout (`ImportError: cannot import name '_pyv8'`), so these are Brimp
  observations, not a side-by-side with the original scripts.

## What was checked

`flows.py` runs the mechanics every example depends on, against `server.py` — a
local stand-in that serves a page, sets a cookie from a page-initiated POST, and
guards an API behind that cookie. It is not a challenge; it answers "can the
runtime run this flow".

```
python benchmark/anti-bot/flows.py        # needs the Brimp Python binding
```

All eleven checks pass:

| Check | Result |
| --- | --- |
| navigate and parse a linked stylesheet | `status=200`; the sheet is still in flight when `get()` returns and `run_pending_tasks()` settles it |
| the page's own challenge script runs | `typeof collect === 'function'` |
| page `fetch` POST receives `Set-Cookie` | `204` |
| session cookie jar holds the cookie | one `session=ok` entry |
| follow-up API request authenticated by it | `200` |
| `route()` blocks a stylesheet by resource type | sheet absent, colour unset |
| `page.click` dispatches pointer events | `pointerdown/mousedown/pointerup/click`, all `isTrusted: true` |
| `page.hover` dispatches `pointermove` | one event |
| an iframe document is reachable from the page | frame text read |
| `run_pending_tasks` drains timers | timer fired |
| `load_html` installs a saved document | text read back |

## The signals a sensor reads

`signals.js` measures the probes `pyv8c/examples/px3/px3.md` lists as the defects
that gave it away, so the same measurements can be compared between Brimp and the
Safari on this machine (Safari is driven through `do JavaScript`, not WebDriver,
which would set `navigator.webdriver`).

```
python benchmark/anti-bot/run-brimp.py | python3 benchmark/anti-bot/run-safari.py
```

| Signal | Brimp | Safari 27 | |
| --- | --- | --- | --- |
| `requestAnimationFrame` rate | 62 fps, 162 frames / 2.5 s | 60 fps (foreground) | matches |
| `performance.now()` resolution | whole ms (0 sub-ms samples) | whole ms | matches |
| `speechSynthesis.getVoices()` | 68, first `Samantha`/`en-US` | 68, same | matches |
| `getBoundingClientRect()` kind | `[object DOMRect]`, real prototype | same | matches |
| `p`/`h1`/`ul` display | `block`/`block`/`block` | same | matches |
| UA margins | `16px`, `21.4375px`, `40px` | same | matches |
| `<link rel=stylesheet>` | parsed, colour applied | same | matches |
| `img` with no `src` | `0×0` | `0×0` | matches |
| empty inline box | `0×16` | `0×18` | **differs** (line height) |
| style object, `for…in` count | 453 | 1465 | **differs** |
| style prototype own members | 2 | 1456 | **differs** |

The three differences were fixed for this run and are recorded in `TODO.md`:
`performance.now()`/`Event.timeStamp` coarsening, 1/64 px length quantisation in
computed style, and the installed voice list. (`emptyInline` and the two style
counts were already failing before; they are the remaining ones.)

## Why a faithful port is still not possible

The examples need APIs Brimp does not have, in rough order of how much of each
script depends on them:

| pyv8c API | Used by | Brimp today |
| --- | --- | --- |
| `page.mouse.move(x, y, curve="human", steps, duration, seed)`, `page.mouse.click(delay=…)`, press-and-hold | Akamai, PX, DataDome | selector-only `click`/`hover`/`tap`; no coordinates, no down/up split |
| `page.goto(url, wait_until="domcontentloaded")`, `page.run_until_idle()` | all eight | no wait conditions; `session.get()` returns before subresources settle and `document.readyState` already reads `complete`, but `page.run_pending_tasks()` does settle them |
| `profile.cookies.header_for(url)` | all eight | `session.cookies` is a list of dicts; build the header yourself |
| `page.net.post/get` (page-context request with cookies) | reese84 | `page.evaluate(fetch(...))` or `session.post` |
| `register_transport`, `CurlCffiTransport` subclass | 5s, reese84 | transport is fixed to curl-impersonate; no hook to observe responses |
| `page.executor.call(fn, timeout=…)` | 5s | `page.evaluate` only |
| `border_box(node)`, `_subtree_nodes(frame.document)` | 5s | `element.getBoundingClientRect()`, frame documents via `iframe.contentDocument` |

## Live runs (network available)

The sandbox reaches the internet through a fake-IP TUN (`198.18.0.0/15` is the
fake range, not a failure), so the example targets can be loaded for real. pyv8c
still cannot run here (its native `_pyv8` is not built), so these are Brimp
observations, not a side-by-side.

```
python benchmark/anti-bot/live/batch.py      # all targets, ~3 min
python benchmark/anti-bot/live/probe.py URL  # one target, with cookie events
python benchmark/anti-bot/live/reese84.py    # the reese84 example, ported
```

| Example | Target | Result |
| --- | --- | --- |
| `5s` (Cloudflare) | steamdb.info/login | `403` interstitial; the orchestrate + Turnstile scripts load and the page text reaches "Verification successful. Waiting for steamdb.info to respond", but no challenge POST is made and no `cf_clearance` appears. pyv8c's version clicks the Turnstile checkbox inside a cross-origin frame; Brimp's input API is selector-only. |
| `akamai` | radissonhotels.com/zh-cn | `200`; the sensor ran and set `_abck`, `bm_sz`, `bm_s`, `bm_ls*`, `ak_bmsc` — `_abck` is not `~0~`, so Akamai did not validate the session. pyv8c's version feeds human mouse curves first, which Brimp cannot do. |
| `datadome` | neimanmarcus.com | `403` interstitial with `datadome` + `dd_testcookie` set, i.e. the same pre-clearance state a first-time browser sees. |
| `f5shape` | southwest.com | `200` with the normal Adobe/consent/SWA cookies; no `f5avr*` or `TS*` sensor cookie. |
| `reese84` | orion.nv.gov | document `200` in ~5 s; the sensor script (`/f-Entell-…/De9r3H4…`) is fetched, no JavaScript error is raised, and no sensor POST follows within 45 s — only `incap_ses_*`/`visid_incap_*`. No `reese84` cookie, so the ported API call never runs. |
| `px3` | skyscanner captcha | page `200`, then an evaluation times out (30 s); the challenge `uuid`/`vid` in the example are from a past session. |

No target produced its pass marker. Two of the six (`akamai`, `datadome`) show the
vendor's sensor running and its pre-cleared cookies being set, which is the state a
real browser reaches before human interaction; the other four stop earlier. The
two examples whose target needs an interaction (Cloudflare's Turnstile checkbox,
Akamai's mouse curves) cannot be driven at all until coordinate input exists.

Also measured on the way: `session.get()` returns while subresources are in flight
and `document.readyState` already reads `complete`, so any port must call
`run_pending_tasks()` in a loop (the examples' `wait_until="domcontentloaded"`
has no equivalent). A page whose script schedules a perpetual microtask loop
(`queueMicrotask(f)` from `f`) makes navigation time out — a real browser freezes
there too, so that is not a divergence; but it means a hang inside a page bundle
(`orion.nv.gov`'s `main.js` timed out once, then loaded in 4.7 s) is
indistinguishable from a slow site without better diagnostics.
