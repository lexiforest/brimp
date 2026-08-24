# web-runtime

The public browser orchestration layer for Brimp.

Each `Page` owns one JavaScriptCore runtime, one canonical Blitz document, its
viewport, wrapper bindings, task queues, and resource-loading state. The crate
provides static content, navigation, classic script scheduling, parser pausing,
events, timers, Promise-based fetch, layout reads, and CPU screenshots.

## Static-page example

```rust
use web_runtime::{Browser, PageOptions};

let browser = Browser::new()?;
let mut page = browser.new_page(
    PageOptions::builder().viewport(1280, 720).build(),
)?;
page.set_content("<div id='box' style='width: 100px'>Hello</div>")?;
page.eval("document.querySelector('#box').style.width = '200px'")?;
assert_eq!(
    page.eval("document.querySelector('#box').getBoundingClientRect().width")?
        .to_number()?,
    200.0,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run the complete static HTML/JavaScript/layout/screenshot example with:

```sh
cargo run -p web-runtime --example mvp1
```

For network navigation, call `Page::goto().await`, then `wait_for_load().await`.
Use `Browser::with_resource_loader` to inject a deterministic or custom
`ResourceLoader`.
