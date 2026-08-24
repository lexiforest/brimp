# screenshot

Optional CPU screenshot rendering for Brimp.

This crate paints a resolved Blitz document through `blitz-paint` and
AnyRender's Vello CPU backend, then encodes the RGBA result as PNG. It creates no
native window and requires neither a GPU nor `blitz-shell`.

The main entry points are:

- `render_png` for rendering a `blitz_dom::BaseDocument`;
- `save_png` for writing encoded bytes;
- `ScreenshotOptions` for output dimensions and full-page behavior.

Most callers should use `web_runtime::Page::screenshot` or
`Page::screenshot_png`, which handles layout and temporary viewport changes.
