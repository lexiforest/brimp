"""Download and verify the pinned WebKit release SDK (never a moving latest release)."""

import argparse
import hashlib
import json
import tarfile
import urllib.request
from pathlib import Path

CONFIG = Path(__file__).with_name("release.json")
TARGETS = {"darwin-arm64": "arm64-macos", "linux-x64": "x86_64-linux-gnu"}


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def verify_sdk(sdk, target, *, release=True):
    config = json.loads(CONFIG.read_text())
    manifest = json.loads((sdk / "manifest.json").read_text())
    if manifest["format"] != 1 or manifest["target"] != TARGETS[target]:
        raise RuntimeError("Wrong WebKit SDK format or target")
    if release and manifest["tag"] != config["tag"]:
        raise RuntimeError("SDK tag does not match tools/webkit/release.json")
    if target == "darwin-arm64" and manifest.get("minimum_macos") != config["minimum_macos"]:
        raise RuntimeError(
            f"WebKit SDK must be rebuilt for macOS {config['minimum_macos']}; "
            f"this SDK targets {manifest.get('minimum_macos')}"
        )
    if (
        target == "linux-x64"
        and manifest.get("runtime_layout_version") != config["runtime_layout_version"]
    ):
        raise RuntimeError(
            "Linux SDK must support relocatable runtime layout version 1; rebuild WebKit"
        )
    if len(manifest["commit"]) != 40 or any(
        c not in "0123456789abcdef" for c in manifest["commit"]
    ):
        raise RuntimeError("SDK must record its full source commit")
    actual = {}
    for path in sorted(sdk.rglob("*")):
        name = path.relative_to(sdk).as_posix()
        if name == "manifest.json":
            continue
        if path.is_symlink():
            if (
                path.readlink().is_absolute()
                or not path.resolve().is_relative_to(sdk.resolve())
                or not path.exists()
            ):
                raise RuntimeError(f"Unsafe SDK link: {name}")
            actual[name] = {"symlink": str(path.readlink())}
        elif path.is_file():
            actual[name] = {"sha256": digest(path)}
    if actual != manifest["files"]:
        raise RuntimeError("SDK file hashes do not match its manifest")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    config = json.loads(CONFIG.read_text())
    version = config["tag"].removeprefix("webkit-v-")
    filename = f"webkit-v{version}.{TARGETS[args.target]}.tar.gz"
    base = f"https://github.com/{config['repository']}/releases/download/{config['tag']}"
    args.output.mkdir(parents=True, exist_ok=False)
    checksums = urllib.request.urlopen(base + "/SHA256SUMS", timeout=60).read().decode()
    expected = {
        name.removeprefix("./"): sha
        for sha, name in (line.split() for line in checksums.splitlines())
    }
    archive = args.output / filename
    urllib.request.urlretrieve(base + "/" + filename, archive)
    if digest(archive) != expected[filename]:
        raise RuntimeError("WebKit release archive checksum mismatch")
    with tarfile.open(archive) as bundle:
        bundle.extractall(args.output, filter="data")
    sdk = args.output / filename.removesuffix(".tar.gz")
    manifest = verify_sdk(sdk, args.target)
    print(
        json.dumps(
            {"sdk": str(sdk.resolve()), "tag": manifest["tag"], "commit": manifest["commit"]}
        )
    )
    # These values are generated locally from the checked configuration/manifest.
    import os

    if path := os.environ.get("GITHUB_ENV"):
        with open(path, "a") as output:
            output.write(f"BRIMP_WEBKIT_SDK={sdk.resolve()}\n")


if __name__ == "__main__":
    main()
