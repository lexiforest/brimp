## General

Do not preserve backward compatibility. Remove obsolete paths instead of adding
compatibility layers, fallbacks, or migrations.

Choose the simplest implementation that fully meets the current requirements.
Avoid speculative abstractions, configuration, and indirection.

Grow the system in layers. Start from the smallest version that works end to end,
and add each new capability on top of a product that already works. Never trade
a working product for unfinished complexity.

Keep components modular and concerns clearly separated.

Prefer established, well-maintained libraries when they reduce overall complexity
or improve reliability. Do not reimplement common functionality without a clear reason.

Lean on the dependencies already in the project before writing your own
implementation or adding packages. Do not assume a library lacks a capability
without checking its documentation and types.

Make architectural decisions for the long term. Do not accept a stopgap that only
works for now and is meant to be replaced later.

Choose the simplest implementation that fully meets the current requirements.

## Files

- For requests to install or use Brimp, read [SKILL.md](SKILL.md). It covers
  published packages and installation verification; source builds are for development.
- Use TODO.md to track the implementation progress and interact with the user.
- Follow but DO NOT EDIT HUMAN.md, it's for the maintainer to edit manually.
- Read README.md and ./docs for the architecture fo this project

## Coding

- Use pytest for python testing
- Prefer playwright over puppeteer in examples and docs
- Use `import brimp` and qualified names such as `brimp.Session` in Python docs and examples.
- Do NOT update the root ./README.md without asking for permission

## Arch & Implementation

This is a runtime for browser JavaScript, which may be badly written, intentionally or not.
Our goal is NOT to make the browser/web API correct. Rather, most of these API calls
are actually unnecessary, you should not assume that a remote external browser script is
perfect, sometimes they have legacy code that relies on the existense of the API, but
don't need the result actually. So you should not assume that the placeholder functions
are optional.

The private `__brimp*` properties must be hidden from external JavaScript files.

- For Python binding, we target Python 3.11 and above.
- For Nodejs binding, we target Node.js 22.19 and above.
