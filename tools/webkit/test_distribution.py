"""Exercise installed Python/Node packages with the input SDK unavailable."""

import argparse
import json
import os
import shutil
import subprocess
import tempfile
import venv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def run(command, directory, environment):
    subprocess.run(
        [str(part) for part in command], cwd=directory, env=environment, check=True, timeout=240
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--dist", type=Path, help="locally built companion packages")
    source.add_argument(
        "--published",
        help="exact companion version from PyPI (Python-only while npm publishing is paused)",
    )
    parser.add_argument("--base-wheels", type=Path, required=True)
    parser.add_argument(
        "--base-node", type=Path, help="Node packages for testing a local companion"
    )
    parser.add_argument("--hide-sdk", type=Path)
    args = parser.parse_args()
    if args.dist and not args.base_node:
        parser.error("--dist requires --base-node")
    sdk = args.hide_sdk.resolve() if args.hide_sdk else None
    if sdk:
        hidden = sdk.with_name(sdk.name + ".distribution-test-hidden")
        if hidden.exists():
            raise RuntimeError(f"Refusing to overwrite {hidden}")
        sdk.rename(hidden)
    try:
        with tempfile.TemporaryDirectory(prefix="brimp-installed-") as directory:
            root = Path(directory)
            environment = {
                key: value
                for key, value in os.environ.items()
                if not key.startswith(("BRIMP_", "DYLD_", "__XPC_DYLD_"))
                and key not in {"PYTHONPATH", "LD_LIBRARY_PATH", "PKG_CONFIG_PATH", "NODE_PATH"}
            }
            environment["XDG_CACHE_HOME"] = str(root / "cache")
            environment["GTK_A11Y"] = "none"
            environment["LIBGL_ALWAYS_SOFTWARE"] = "1"
            python_root = root / "venv"
            venv.EnvBuilder(with_pip=True, symlinks=True).create(python_root)
            python = python_root / "bin/python"
            wheels = list(args.base_wheels.resolve().glob("*.whl"))
            # Install only Brimp explicitly: pip must resolve the required companion.
            if args.dist:
                dependency_source = ["--find-links", args.dist.resolve()]
            else:
                constraints = root / "constraints.txt"
                constraints.write_text(f"brimp-webkit=={args.published}\n")
                dependency_source = ["--constraint", constraints]
            # CI passes one base wheel for this platform, so pip cannot select a source checkout.
            run(
                [python, "-m", "pip", "install", *dependency_source, "pytest", *wheels],
                root,
                environment,
            )
            run([python, "-m", "pip", "check"], root, environment)
            run(
                [
                    python,
                    "-c",
                    "import brimp; s=brimp.Session(engine='webkit'); p=s.new_page(); assert p.evaluate('6*7') == 42; s.close()",
                ],
                root,
                environment,
            )
            executable = subprocess.check_output(
                [python, "-c", "import brimp_webkit; print(brimp_webkit.executable_path())"],
                cwd=root,
                env=environment,
                text=True,
            ).strip()
            environment["BRIMP_WEBKIT_EXECUTABLE"] = executable
            shutil.copy2(
                ROOT / "bindings/python/tests/test_browser_backend.py", root / "test_backend.py"
            )
            run([python, "-m", "pytest", root / "test_backend.py", "-v"], root, environment)
            if args.published:
                print(
                    json.dumps(
                        {
                            "python": "passed",
                            "node": "not run: npm publishing is paused",
                            "sdk_hidden": sdk is not None,
                            "published_companion": args.published,
                            "automatic_discovery": True,
                        }
                    )
                )
                return
            # Use another extraction directory to test Node's independent installation path.
            environment["XDG_CACHE_HOME"] = str(root / "node-cache")
            (root / "package.json").write_text('{"private":true,"type":"module"}\n')
            packages = [*args.base_node.resolve().glob("*.tgz"), *args.dist.resolve().glob("*.tgz")]
            run(
                [
                    "npm",
                    "install",
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                    "--omit=optional",
                    *packages,
                ],
                root,
                environment,
            )
            run(
                [
                    "node",
                    "--input-type=module",
                    "-e",
                    "import {createSession} from '@brimp/brimp'; const s=await createSession({engine:'webkit'}); try { const p=await s.newPage(); if(await p.evaluate('6*7')!==42) throw Error('evaluation failed'); } finally { await s.close(); }",
                ],
                root,
                environment,
            )
            executable = subprocess.check_output(
                ["node", "-e", "console.log(require('@brimp/webkit').executablePath())"],
                cwd=root,
                env=environment,
                text=True,
            ).strip()
            environment["BRIMP_WEBKIT_EXECUTABLE"] = executable
            contract = (
                (ROOT / "bindings/node/test/browser-backend.test.mjs")
                .read_text()
                .replace("'../index.js'", "'@brimp/brimp'")
            )
            (root / "backend.test.mjs").write_text(contract)
            run(["node", "--test", root / "backend.test.mjs"], root, environment)
            print(
                json.dumps(
                    {
                        "python": "passed",
                        "node": "passed",
                        "sdk_hidden": sdk is not None,
                        "published_companion": args.published,
                        "automatic_discovery": True,
                    }
                )
            )
    finally:
        if sdk:
            hidden.rename(sdk)


if __name__ == "__main__":
    main()
