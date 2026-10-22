use brimp_runtime::dom::BrowserDocument;
use std::{hint::black_box, time::Instant};
fn main() {
    let count = 3000;
    let html = format!("<!doctype html><title>bench</title><main>{}</main>",
        (0..count).map(|i| format!("<article class='item' data-n='{i}'><h2>Title {i}</h2><p>Some text and <b>markup</b>.</p></article>")).collect::<String>());
    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        let doc = BrowserDocument::parse(&html);
        let nodes = doc.query_selector_all("main > article.item").unwrap();
        assert_eq!(nodes.len(), count);
        for node in nodes.iter().take(100) {
            doc.set_inline_style_css(*node, "color: red; margin: 2px;");
        }
        black_box(doc.outer_html());
    }
    println!(
        "{}",
        serde_json::json!({"iterations":iterations,"elements_per_document":count,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0})
    );
}
