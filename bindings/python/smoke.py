"""Exercise an installed wheel without importing repository fixtures."""

from brimp import Session, plugins


with Session(plugins=[plugins.canvas(), plugins.extraction()]) as session:
    with session.new_page() as page:
        assert page.evaluate("6 * 7") == 42
        assert page.evaluate("localStorage.x='installed'; localStorage.x") == "installed"
        assert page.evaluate(
            "Array.from(document.createElement('canvas').getContext('2d').getImageData(0,0,1,1).data)"
        ) == [0, 0, 0, 0]
        page.evaluate(
            "document.body.innerHTML='<article><h1>Package check</h1>"
            "<p>Extracted from the installed package.</p></article>'; null"
        )
        result = page.extract(content_selector="article")
        assert "installed package" in result["contentMarkdown"]
print("ok")
