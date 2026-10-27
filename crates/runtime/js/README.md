# Runtime JavaScript

- `internal/` contains host bridges, module loading, plugin installation,
  diagnostics, evaluation helpers and profile setup.
- `web_apis/` implements the browser-facing DOM and shared web API surfaces.
- `plugins/` owns browser subsystems and host integrations.
- `vendor/` contains dependency adapters, build tooling and generated bundles.

The private profile registry and installation controller live in `internal/`.
Core interface defaults and capture recipes live in `web_apis/profile.js`;
subsystem plugins register their own values and recipes. Most DOM files share a closure;
this directory layout does not change their loading order or lexical scope.

From the repository root, install the pinned development tools and format authored
runtime sources:

```sh
npm ci --ignore-scripts
npm run format:runtime
npm run format:runtime:check
```

Prettier uses `.prettierrc.json` in this directory: two-space indentation,
single quotes and a 100-column print width. CI runs the check command.
The repository `.prettierignore` excludes generated vendor bundles and
`web_apis/css_property_names.js`. Format the authored vendor adapters,
then rebuild bundles with `npm run build --prefix crates/runtime/js/vendor`.

Plugin installers and the bridge/diagnostics/plugin controllers are named function
declarations embedded as expressions by their loaders. Unlike standalone arrow
function statements, these remain embeddable after ordinary Prettier formatting.
The DOM files share one lexical scope; `bindings.rs` supplies the enclosing
closure so each source file remains independently parseable by the formatter.
