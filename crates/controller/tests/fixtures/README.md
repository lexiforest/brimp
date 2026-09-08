# CDP deterministic fixtures

Serve this directory with `python3 -m http.server 8000 --directory
`tests/fixtures`. The controller acceptance suite uses these
pages for protocol behavior; public-site runs remain performance/soak tests.

- `lifecycle.html`: DOMContentLoaded/load ordering and a late marker.
- `never-complete.html`: a deliberately pending subresource; navigation must
  still acknowledge immediately.
- `frames.html`: same-origin child-frame discovery.
- `values.html`: by-value JavaScript types and rejected promises.
- `dialogs.html`: alert/confirm/prompt policy.
- `input.html`: trusted mouse and keyboard result recording.
- `screenshot.html`: deterministic colors, overflow, and scroll geometry.
- `redirect.py`: deterministic redirect, response, and failed-resource routes.
