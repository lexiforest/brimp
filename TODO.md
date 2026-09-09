# Rename crawl concurrency option

- [x] Replace --workers with --concurrency in parsing, help, internal names, docs, and the crawl integration test; keep the default at 2.
- [x] Pass the crawl integration test, verify rejection of --workers and zero concurrency, and check help, formatting, diffs, and the docs build.

# Simplify architecture documentation

- [x] Condense ARCH.md into the development guide and link to detailed subsystem and CDP documentation.
- [x] Remove the standalone architecture file and duplicated explanations.
- [x] Build all 17 documentation pages and verify internal links, the architecture anchor, and clean diffs.

# Fix commit formatting check after crate consolidation

- [x] Remove the obsolete runtime example target and point its README command to the existing integration test.
- [x] Verify the pre-commit formatting check passes (`sh .githooks/pre-commit`).
