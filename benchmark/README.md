# Benchmarks

The [native benchmarks](native/README.md) measure DOM
parse/select/mutate/serialize and complete Python/Node
request → script → query/edit → result workflows using local fixtures.
The documentation includes reproduction commands, elapsed time and peak RSS.

The [network comparison](network/README.md) measures the runtime resource adapter
against direct brimp-net requests on local HTTP/1.1 workloads.

The [compatibility corpus](compatibility/README.md) compares browser-library
behavior across Brimp, Chromium, jsdom and happy-dom. It includes a targeted
[DOM coverage audit](compatibility/coverage/README.md). These comparisons measure
functional compatibility rather than speed.

Correctness is covered by the focused Rust, Python and Node regression suites.
See the [development guide](../docs/src/content/docs/development.md) for commands.
