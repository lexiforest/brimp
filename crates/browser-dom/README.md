# browser-dom

The canonical Blitz-backed document boundary for Brimp.

`BrowserDocument` owns the page's sole DOM tree and exposes focused operations
for traversal, selectors, mutation, style inspection, layout geometry, and
viewport updates. It does not maintain a parallel wrapper tree.

`HtmlParserSession` adds incremental html5ever parsing on the same Blitz tree.
It yields at parser-inserted scripts so the browser runtime can pause, execute
JavaScript, and resume without losing DOM mutations.

## Example

```rust
use browser_dom::BrowserDocument;

let document = BrowserDocument::parse(
    "<!doctype html><html><body><div id='hello'>Hello</div></body></html>",
);
let node_id = document.get_element_by_id("hello").unwrap();
assert_eq!(document.node(node_id).unwrap().text_content(), "Hello");
```

Run the parser example with:

```sh
cargo run -p browser-dom --example parse_html
```
