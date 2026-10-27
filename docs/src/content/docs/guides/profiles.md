---
title: Capture browser profiles
description: Measure values in a live browser and supply them to installed runtime interfaces.
---

A profile supplies values to core DOM/browser interfaces and selected subsystem
plugins. Each interface owns its defaults, validation rules and browser capture
recipe. There is no separate complete schema to fill in by hand.

## Capture from your browser

```sh
brimp profile capture --output browser-profile.json
```

Open the printed localhost URL in the browser you want to measure. Keep the window
at the size you want to capture, then click **Capture**. The page displays each
result and submits the profile to the local command, which validates it against
the selected interfaces and saves it. The command exits after a successful save.
The page also offers **Download profile**.

Use `--overwrite` to replace an existing output. The server binds to `127.0.0.1`;
`--port 8766` chooses a port and `--timeout 5m` changes the waiting deadline.
Ctrl-C stops the command. A rejected submission leaves the destination unchanged
and the server available for another attempt.

The default browser plugins participate automatically. Repeated `--plugin` flags
select a complete set, and `--no-plugins` selects core only. A JavaScript plugin
file can register its own profile values and capture recipe:

```sh
brimp profile capture --plugin ./my-plugin.js --output browser-profile.json
```

Export the wrapper and its script for independent hosting:

```sh
brimp profile capture --directory capture-kit --no-serve --output browser-profile.json
python3 -m http.server 8766 --bind 127.0.0.1 --directory capture-kit
```

Visit `http://127.0.0.1:8766`, click **Capture**, then download the profile. An
independently hosted kit does not submit to the CLI. Without `--no-serve`, the
command both exports the kit and starts its own saving endpoint.

## Load and inspect a profile

```sh
brimp fetch https://example.com --profile browser-profile.json --eval 'navigator.userAgent'
```

```javascript
import {readFile} from 'node:fs/promises';
import {createSession} from '@brimp/brimp';
const profile = JSON.parse(await readFile('browser-profile.json', 'utf8'));
const session = await createSession({profile});
try {
  const page = await session.newPage();
  console.log(await page.profileReport());
  console.log(await page.profileContract());
} finally {
  await session.close();
}
```

```python
import json
from pathlib import Path
from brimp import Session

profile = json.loads(Path('browser-profile.json').read_text())
with Session(profile=profile) as session:
    page = session.new_page()
    print(page.profile_report())
    print(page.profile_contract())
```

Rust pages expose `profile_contract()` and `profile_report()`. All bindings return
the same document keys, including custom plugin names; Python does not rename
keys inside these documents. Contracts include the registered interfaces, their
owners and versions, field defaults/constraints and serialized capture recipes.
Reports include `defaultsUsed` paths and the supplied `capture` metadata. These
are host APIs; neither the registry nor a private bridge is exposed to page scripts.

Profiles are partial. Missing object fields use interface defaults; supplied
arrays replace the default arrays. Unknown fields, invalid types and scopes for
unloaded interfaces are rejected. For example, an audio profile requires the audio
plugin to be selected when loading it. A small explicit override is also valid:

```json
{"values":{"audio":{"sample_rate":48000,"max_channel_count":8}}}
```

Native HTTP identity and JavaScript navigator identity use the same resolved core
values. Host identity options can override individual identity fields. Plugin
options remain explicit runtime configuration and can override their corresponding
profile values.

## Understand the measurements

The saved envelope has `format: "brimp-profile"`, `version: 1`, `values`, and
`capture` metadata. Metadata records the tool version, capture time, user agent,
selected plugins, interface recipe versions, result statuses and defaulted paths.
Only successful measurements enter `values`:

| Status | Meaning |
| --- | --- |
| `captured` | A measured value passed the interface's validation. |
| `absent` | The browser did not expose the requested value. |
| `permission-required` | The browser denied access with a permission/security error. |
| `failed` | The probe threw or returned a value that could not be accepted. |
| `unavailable` | No page-level measurement exists for this setting. |

Absence is metadata, not an instruction to remove APIs. Runtime compatibility
placeholders and plugin selection are independent of capture results. Unmeasured
settings use defaults and appear in the load report.

Font capture tests candidate families using text-width comparisons and measures
average advances; it does not enumerate every installed font or reproduce shaping.
Canvas/WebGL/audio capture requires creating those browser objects. Media device
capture does not request microphone or camera permission, so labels may be empty.
HTTP header defaults and TLS impersonation cannot be inferred reliably from page
JavaScript and remain explicitly unmeasured. Choose the transport impersonation
separately when appropriate; capturing a browser does not capture its TLS stack.

## Define values in a plugin

An installer receives `profile` alongside `global`, `host` and other plugin context.
Use the returned values in the implementation so defaults, overrides and probes
stay together:

```javascript
({global, profile}) => {
  const values = profile.values('example', {answer: 42}, {
    version: 1,
    fields: {answer: {minimum: 0, integer: true}},
    capture: ({read}) => ({answer: read(() => navigator.hardwareConcurrency)}),
  });
  global.exampleAnswer = () => values.answer;
}
```

Types are inferred from defaults. Field hints can specify `type`, `nullable`,
`items`, `additionalProperties`, `minimum`, `maximum`, `integer`, `enum` and
`nonempty`. An empty array needs an explicit `items` hint if its items require
validation. A null default needs a type hint when it represents an optional typed
value. Scope names must be unique across the loaded core and plugins.

Recipes receive `{read, readRecord, defaults}`. `read(() => value)` classifies access failures
and rejects values that cannot be represented as JSON; it also accepts an async
getter. An async recipe can await those results. `readRecord({key: getter, ...})`
collects independent object entries and retains their individual failure statuses.
Asynchronous probe/field waits time out after ten seconds so later interfaces can proceed. Return an object mapping registered
field names to results. Omit settings that cannot be observed in a browser.
Increment `version` when the meaning of the interface or its recipe changes.
On load, recorded interface owners and versions must match the selected definitions;
a mismatch requests a fresh capture instead of silently interpreting old values.

A recipe must be self-contained: its source is exported without the installer,
closed-over variables, host handlers or private bridge. Put required helpers inside
the recipe. The exported kit contains capture functions, defaults and validation,
not the runtime plugin implementations. A captured value must be serializable;
functions, BigInt, undefined members and non-finite numbers are not silently coerced.
