"""Brimp and its companion have independent versions and tag namespaces."""

import json
import shutil
import sys

import check_versions
import pytest


@pytest.fixture
def checkout(tmp_path, monkeypatch):
    for name in (
        "Cargo.toml",
        "bindings/python/pyproject.toml",
        "bindings/python/Cargo.toml",
        "bindings/python/LICENSE",
        "bindings/node/package.json",
        "bindings/node/LICENSE",
        "tools/webkit/release.json",
        "LICENSE",
    ):
        destination = tmp_path / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(check_versions.ROOT / name, destination)
    monkeypatch.setattr(check_versions, "ROOT", tmp_path)
    return tmp_path


def test_brimp_bump_keeps_companion_version(checkout, monkeypatch, capsys):
    cargo = checkout / "Cargo.toml"
    original = json.loads((checkout / "bindings/node/package.json").read_text())["version"]
    cargo.write_text(cargo.read_text().replace(f'version = "{original}"', 'version = "9.8.7"', 1))
    node_path = checkout / "bindings/node/package.json"
    node = json.loads(node_path.read_text())
    node["version"] = "9.8.7"
    node["optionalDependencies"] = dict.fromkeys(node["optionalDependencies"], "9.8.7")
    node_path.write_text(json.dumps(node))
    monkeypatch.setattr(sys, "argv", ["check_versions.py", "--tag", "v9.8.7"])
    assert check_versions.main() == 0
    assert capsys.readouterr().out.strip() == "9.8.7"
    companion = json.loads((checkout / "tools/webkit/release.json").read_text())["version"]
    monkeypatch.setattr(
        sys, "argv", ["check_versions.py", "--package", "webkit", "--tag", f"webkit-v{companion}"]
    )
    assert check_versions.main() == 0
    assert capsys.readouterr().out.strip() == companion


@pytest.mark.parametrize(
    "package,tag", [("brimp", "webkit-v0.1.0"), ("webkit", "v0.1.2"), ("webkit", "webkit-v9.9.9")]
)
def test_wrong_release_tag_is_rejected(checkout, monkeypatch, package, tag):
    monkeypatch.setattr(sys, "argv", ["check_versions.py", "--package", package, "--tag", tag])
    with pytest.raises(RuntimeError, match="release tag"):
        check_versions.main()


@pytest.mark.parametrize("binding", ["python", "node"])
def test_companion_pin_mismatch_is_rejected(checkout, monkeypatch, binding):
    if binding == "python":
        path = checkout / "bindings/python/pyproject.toml"
        version = json.loads((checkout / "tools/webkit/release.json").read_text())["version"]
        path.write_text(path.read_text().replace(f"brimp-webkit=={version}", "brimp-webkit==9.9.9"))
    else:
        path = checkout / "bindings/node/package.json"
        node = json.loads(path.read_text())
        node["dependencies"]["@brimp/webkit"] = "9.9.9"
        path.write_text(json.dumps(node))
    monkeypatch.setattr(sys, "argv", ["check_versions.py"])
    with pytest.raises(RuntimeError, match="companion version"):
        check_versions.main()


@pytest.mark.parametrize("binding", ["python", "node"])
def test_optional_companion_is_rejected(checkout, monkeypatch, binding):
    if binding == "python":
        path = checkout / "bindings/python/pyproject.toml"
        text = path.read_text()
        version = json.loads((checkout / "tools/webkit/release.json").read_text())["version"]
        text = text.replace(f'dependencies = ["brimp-webkit=={version}"]\n', "")
        path.write_text(
            text + f'\n[project.optional-dependencies]\nwebkit = ["brimp-webkit=={version}"]\n'
        )
    else:
        path = checkout / "bindings/node/package.json"
        node = json.loads(path.read_text())
        node["optionalDependencies"]["@brimp/webkit"] = node["dependencies"]["@brimp/webkit"]
        path.write_text(json.dumps(node))
    monkeypatch.setattr(sys, "argv", ["check_versions.py"])
    with pytest.raises(RuntimeError, match="Required.*dependency"):
        check_versions.main()
