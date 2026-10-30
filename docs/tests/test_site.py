"""Verify built documentation links and generated API coverage without native SDKs."""

import ast
from html.parser import HTMLParser
from pathlib import Path
import re
from urllib.parse import unquote, urljoin, urlsplit

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "docs/dist"


class Page(HTMLParser):
    def __init__(self, content):
        super().__init__()
        self.ids, self.links = set(), []
        self.feed(content)

    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if "id" in values:
            self.ids.add(values["id"])
        for name in ("href", "src"):
            if values.get(name):
                self.links.append(values[name])


def test_internal_links():
    assert (SITE / "index.html").is_file(), "Build docs before checking the generated site"
    pages = {p: Page(p.read_text()) for p in SITE.rglob("*.html")}
    failures = []
    for file, page in pages.items():
        route = "/" + file.relative_to(SITE).as_posix()
        for link in page.links:
            parts = urlsplit(urljoin(route, link))
            if parts.scheme or parts.netloc:
                continue
            target = SITE / unquote(parts.path).lstrip("/")
            if target.is_dir():
                target /= "index.html"
            if not target.is_file():
                failures.append(f"{file.relative_to(SITE)}: missing {link}")
            elif parts.fragment and target in pages:
                anchor = unquote(parts.fragment)
                if anchor not in pages[target].ids:
                    failures.append(f"{file.relative_to(SITE)}: missing anchor {link}")
    assert not failures, "\n".join(failures)


def test_python_reference_has_public_members():
    reference = (SITE / "reference/python/index.html").read_text()
    tree = ast.parse((ROOT / "bindings/python/python/brimp/__init__.py").read_text())
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and not node.name.startswith("_"):
            assert f'id="brimp.{node.name}"' in reference
            for child in node.body:
                if isinstance(child, ast.FunctionDef) and not child.name.startswith("_"):
                    assert f'id="brimp.{node.name}.{child.name}"' in reference
    # Inherited verbs and dynamically assigned response metadata need explicit coverage.
    for symbol in [
        "Session.get",
        "Page.post",
        "Response.status_code",
        "Response.html",
        "Route.stage",
    ]:
        assert f'id="brimp.{symbol}"' in reference
    plugins = (SITE / "reference/python/plugins/index.html").read_text()
    assert "brimp.plugins.host_storage" in plugins
    errors = (SITE / "reference/python/errors/index.html").read_text()
    assert "StaleHandle" in errors and "TransportError" in errors


def test_node_and_cli_references():
    node = SITE / "reference/node"
    assert (node / "functions/createSession.html").is_file()
    page = (node / "classes/Page.html").read_text()
    for member in [
        "evaluate",
        "get",
        "route",
        "advanceTime",
        "stream",
        "configureNetwork",
        "peekDiagnostics",
        "summarizeDiagnostics",
    ]:
        assert f'id="{member.lower()}"' in page or f'id="{member}"' in page
    cli = (SITE / "api/cli/index.html").read_text()
    for flag in [
        "--actions",
        "--routes",
        "--profile",
        "--stream",
        "--clock",
        "--trace-report",
        "--trace-mode",
    ]:
        assert flag in cli
    assert "Generated from the CLI" in cli


def test_architecture_pngs():
    for name in ["runtime-layers", "page-lifecycle"]:
        png = SITE / f"diagrams/{name}.png"
        assert png.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
        assert (ROOT / f"docs/diagrams/{name}.svg").is_file()


def test_removed_api_paths_do_not_return_in_guides():
    bad = re.compile(
        r"brimp\.get\(|\blaunch\(|\.screenshot\(|brimp (?:eval|cdp|serve)\b|--timeout-ms|profileJson|profile_json"
    )
    for source in (ROOT / "docs/src/content/docs").rglob("*"):
        if source.suffix in (".md", ".mdx"):
            assert not bad.search(source.read_text()), str(source)
