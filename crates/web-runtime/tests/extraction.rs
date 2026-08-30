use web_runtime::{Browser, ExtractionOptions, PageOptions};

#[test]
fn defuddle_extracts_markdown_from_the_live_dom_without_leaking_a_global() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        r#"<!doctype html>
        <html lang="en">
          <head>
            <title>Agent Browsers</title>
            <meta name="description" content="A useful article">
          </head>
          <body>
            <nav>Navigation that should not be the article.</nav>
            <article id="story">
              <h1>Agent Browsers</h1>
              <p>This paragraph was rendered in the live document.</p>
            </article>
          </body>
        </html>"#,
    )
    .unwrap();
    page.eval("document.querySelector('p').textContent += ' Updated.'")
        .unwrap();
    let before = page.document().outer_html();

    let extracted = page
        .extract(ExtractionOptions {
            content_selector: Some("#story".into()),
            ..ExtractionOptions::default()
        })
        .unwrap();

    assert_eq!(extracted.title, "Agent Browsers");
    assert!(
        extracted
            .content_markdown
            .as_deref()
            .unwrap()
            .contains("This paragraph was rendered in the live document. Updated.")
    );
    assert_eq!(page.document().outer_html(), before);
    assert_eq!(
        page.eval("typeof globalThis.Defuddle")
            .unwrap()
            .to_string()
            .unwrap(),
        "undefined"
    );
}

#[test]
fn defuddle_preserves_article_structure_metadata_and_extraction_options() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        r#"<!doctype html>
        <html><head>
          <title>Structured article</title>
          <meta name="author" content="Brimp Agent">
          <meta name="description" content="Compatibility fixture">
        </head><body><main id="content">
          <h1>Structured article</h1>
          <p>Read the <a href="https://example.test/guide">guide</a>.</p>
          <p>This compatibility fixture contains enough prose to exercise the normal article path without Defuddle falling back to its sparse index-page mode. It verifies that structured browser content remains readable after the extractor clones and cleans the live document, while preserving code blocks, links, tables, metadata, and user-selected extraction behavior.</p>
          <pre><code>session.extract()</code></pre>
          <table><tr><th>Surface</th><th>Status</th></tr><tr><td>DOM</td><td>live</td></tr></table>
          <img src="https://example.test/hero.png" alt="Hero">
          <p hidden style="display:none">This must not be included.</p>
        </main></body></html>"#,
    )
    .unwrap();

    let extracted = page
        .extract(ExtractionOptions {
            content_selector: Some("#content".into()),
            remove_images: true,
            language: Some("fr".into()),
            debug: true,
        })
        .unwrap();
    let markdown = extracted.content_markdown.unwrap();

    assert_eq!(extracted.title, "Structured article");
    assert_eq!(extracted.description, "Compatibility fixture");
    assert_eq!(extracted.author, "Brimp Agent");
    assert!(markdown.contains("session.extract()"), "{markdown}");
    assert!(markdown.contains("Surface"), "{markdown}");
    assert!(markdown.contains("/guide"), "{markdown}");
    assert!(!markdown.contains("hero.png"), "{markdown}");
    assert!(
        !markdown.contains("This must not be included"),
        "{markdown}"
    );
    assert!(extracted.debug.is_some());
}
