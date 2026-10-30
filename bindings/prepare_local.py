"""Copy locally built native extensions into their development package paths."""

import argparse
import os
from pathlib import Path
import shutil
import sys
import tempfile


def copy_native(source, destination):
    # Replace the inode: overwriting a previously loaded Mach-O file can leave
    # macOS with cached code-signature pages from the old build.
    with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as file:
        temporary = Path(file.name)
    try:
        shutil.copy2(source, temporary)
        temporary.replace(destination)
    finally:
        temporary.unlink(missing_ok=True)


def prepare(root, release=False, only="both"):
    build = root / "target" / ("release" if release else "debug")
    runtime_libraries = []
    if sys.platform == "win32":
        # Python 3.8+ does not use PATH to resolve extension dependencies.
        # Keep the SDK DLLs beside the development extensions, as Node packages do.
        directories = os.environ.get("BRIMP_NATIVE_RUNTIME_DIRS", "").split(os.pathsep)
        if not all(directories):
            raise RuntimeError("prepare the native SDKs before copying Windows bindings")
        runtime_libraries = [
            path for directory in directories for path in sorted(Path(directory).glob("*.dll"))
        ]
        names = {path.name.lower() for path in runtime_libraries}
        if not {"javascriptcore.dll", "libcurl-impersonate.dll"} <= names:
            raise RuntimeError("native runtime directories must contain the JSC and curl DLLs")

    if sys.platform == "darwin":
        libraries = ("lib_brimp.dylib", "libbrimp_node.dylib")
    elif sys.platform == "win32":
        libraries = ("_brimp.dll", "brimp_node.dll")
    else:
        libraries = ("lib_brimp.so", "libbrimp_node.so")
    destinations = (
        root
        / "bindings/python/python/brimp"
        / ("_brimp.pyd" if sys.platform == "win32" else "_brimp.abi3.so"),
        root / "bindings/node/brimp.node",
    )
    selected = (
        [(libraries[0], destinations[0])]
        if only == "python"
        else [(libraries[1], destinations[1])]
        if only == "node"
        else list(zip(libraries, destinations))
    )
    for source, destination in selected:
        path = build / source
        if not path.is_file():
            raise SystemExit(f"missing build artifact: {path}")
        copy_native(path, destination)
        for library in runtime_libraries:
            copy_native(library, destination.parent / library.name)
        print(destination.relative_to(root))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--only", choices=("both", "python", "node"), default="both")
    args = parser.parse_args()
    prepare(Path(__file__).resolve().parent.parent, args.release, args.only)


if __name__ == "__main__":
    main()
