"""Build the WebKit host against a selected macOS checkout or Linux GTK SDK."""

import argparse
import json
import os
import platform
import shlex
import struct
import subprocess
import tempfile
from pathlib import Path

from download import CONFIG


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--webkit", type=Path, help="built macOS WebKit checkout")
    source.add_argument("--sdk", type=Path, help="extracted WebKit release SDK")
    parser.add_argument("--output", type=Path, default=Path("target/webkit"))
    args = parser.parse_args()
    if platform.system() == "Linux":
        if not args.sdk:
            parser.error("Linux requires --sdk pointing to the installed WebKitGTK SDK")
        build_linux(args, parser)
        return
    if platform.system() != "Darwin":
        parser.error("WebKit hosts support macOS and Linux")
    if args.sdk:
        build_macos(args.sdk.resolve() / "lib", args.output.resolve(), parser)
        return
    revision = Path(__file__).with_name("REVISION").read_text().strip()
    actual = subprocess.check_output(
        ["git", "-C", str(args.webkit), "rev-parse", "HEAD"], text=True
    ).strip()
    if actual != revision:
        parser.error(f"Expected WebKit revision {revision}, found {actual}")
    frameworks = args.webkit.resolve() / "WebKitBuild/Release"
    build_macos(frameworks, args.output.resolve(), parser)


def build_macos(frameworks, output, parser):
    header = frameworks / "WebKit.framework/PrivateHeaders/_WKAutomationSessionLocalConnection.h"
    if not header.exists():
        parser.error(f"Build the WebKit checkout with the local connection SPI first: {header}")
    output.mkdir(parents=True, exist_ok=True)
    binary = output / "brimp-webkit-host"
    subprocess.run(
        [
            "xcrun",
            os.environ.get("CXX", "clang++"),
            "-std=c++20",
            "-mmacosx-version-min=" + json.loads(CONFIG.read_text())["minimum_macos"],
            "-fobjc-arc",
            "-O2",
            "-g",
            "-Wno-deprecated-declarations",
            "-F",
            str(frameworks),
            "-iframework",
            str(frameworks),
            "-framework",
            "AppKit",
            "-framework",
            "WebKit",
            str(Path(__file__).with_name("main.mm")),
            "-o",
            str(binary),
        ],
        check=True,
    )
    launcher = output / "brimp-webkit"
    launcher.write_text(
        "#!/bin/sh\n"
        f"export DYLD_FRAMEWORK_PATH={shlex.quote(str(frameworks))}\n"
        f"export DYLD_LIBRARY_PATH={shlex.quote(str(frameworks))}\n"
        f'exec {shlex.quote(str(binary))} "$@"\n'
    )
    launcher.chmod(0o755)
    verify(launcher, frameworks, parser)
    print(launcher)


def build_linux(args, parser):
    sdk = args.sdk.resolve()
    library = sdk / "lib/libwebkitgtk-6.0.so"
    if not library.is_file():
        parser.error(f"Missing WebKitGTK library: {library}")
    environment = dict(os.environ)
    environment["PKG_CONFIG_PATH"] = str(sdk / "lib/pkgconfig")
    prefix = subprocess.check_output(
        ["pkg-config", "--variable=prefix", "webkitgtk-6.0"], env=environment, text=True
    ).strip()
    if Path(prefix).resolve() != sdk:
        parser.error(f"pkg-config selected {prefix}, expected {sdk}")
    flags = shlex.split(
        subprocess.check_output(
            ["pkg-config", "--cflags", "--libs", "webkitgtk-6.0"], env=environment, text=True
        )
    )
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    binary = output / "brimp-webkit-host"
    subprocess.run(
        [
            os.environ.get("CXX", "clang++"),
            "-std=c++20",
            "-O2",
            "-g",
            "-Wall",
            "-Wextra",
            str(Path(__file__).with_name("main-gtk.cpp")),
            "-o",
            str(binary),
            *flags,
            "-ldl",
            "-pthread",
        ],
        env=environment,
        check=True,
    )
    launcher = output / "brimp-webkit"
    launcher.write_text(
        "#!/bin/sh\n"
        f"export LD_LIBRARY_PATH={shlex.quote(str(sdk / 'lib'))}"
        "${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}\n"
        f"export BRIMP_WEBKIT_ROOT={shlex.quote(str(sdk))}\n"
        f"export WEBKIT_EXEC_PATH={shlex.quote(str(sdk / 'libexec/webkitgtk-6.0'))}\n"
        f"export WEBKIT_INJECTED_BUNDLE_PATH={shlex.quote(str(sdk / 'lib/webkitgtk-6.0/injected-bundle'))}\n"
        f"export BRIMP_WEBKIT_LIBRARY={shlex.quote(str(library))}\n"
        f'exec {shlex.quote(str(binary))} "$@"\n'
    )
    launcher.chmod(0o755)
    # A display is a host prerequisite. Use a private virtual display only for
    # this build verification; the launcher itself execs the host directly so
    # its PID and lifetime are owned by the Rust transport.
    with tempfile.TemporaryDirectory(prefix="brimp-webkit-verify-") as temporary:
        environment.update(XDG_RUNTIME_DIR=temporary, GTK_A11Y="none", LIBGL_ALWAYS_SOFTWARE="1")
        verify(launcher, sdk, parser, environment, ["dbus-run-session", "--", "xvfb-run", "-a"])
    print(launcher)


def verify(launcher, expected, parser, environment=None, prefix=()):
    messages = b""
    for identifier, method in [(1, "Brimp.hello"), (2, "Brimp.close")]:
        payload = json.dumps({"id": identifier, "method": method}).encode()
        messages += struct.pack(">I", len(payload)) + payload
    result = subprocess.run(
        [*prefix, str(launcher)],
        input=messages,
        capture_output=True,
        timeout=30,
        check=False,
        env=environment,
    )
    if result.returncode:
        parser.error(
            f"Helper exited with status {result.returncode}: "
            + result.stderr.decode(errors="replace")
        )
    messages = memoryview(result.stdout)
    hello = None
    while len(messages) >= 4:
        size = struct.unpack(">I", messages[:4])[0]
        message = json.loads(bytes(messages[4 : 4 + size]))
        if message.get("id") == 1:
            hello = message.get("result")
        messages = messages[4 + size :]
    if not hello:
        parser.error("Helper exited without a successful handshake")
    if hello["protocol"] != 1 or not Path(hello["framework"]).resolve().is_relative_to(
        expected.resolve()
    ):
        parser.error(f"Helper loaded an unexpected framework: {hello}")


if __name__ == "__main__":
    main()
