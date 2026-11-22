# Optional Ubuntu 24.04 packaging checks

Optional container setup for checking portable Brimp and WebKit packages.
For native Ubuntu 26.04 development, use [README.md](README.md).
Use an Ubuntu 24.04 or 26.04 **x86_64** host. Builds run in Ubuntu 24.04 containers;
the host distribution does not determine the output's glibc requirement.
Allow at least 16 GiB RAM and 60 GiB free disk; 32–64 GiB RAM is preferable for
WebKit. Downloads, CMake/Ninja outputs, Cargo outputs and ccache persist in the
work directory, so subsequent runs are incremental.

## Set up a new node

Connect with SSH and clone both repositories. Select the WebKit branch, tag or
commit you intend to build; the scripts never switch or pull either checkout.

```sh
apt-get update
apt-get install -y git
mkdir -p /opt/brimp/src
git clone https://github.com/lexiforest/brimp.git /opt/brimp/src/brimp
git clone --depth 1 --branch brimp https://github.com/lexiforest/brimp-webkit.git /opt/brimp/src/WebKit
cd /opt/brimp/src/brimp
bash tools/linux/setup-packaging.sh
bash tools/linux/run-packaging.sh doctor /opt/brimp/src/WebKit /opt/brimp/linux
bash tools/linux/run-packaging.sh all /opt/brimp/src/WebKit /opt/brimp/linux
```

These examples use root, as on a newly provisioned node. For a normal login, use
`sudo` for setup and Docker commands (or configure Docker access yourself).
The containers **build and test as an unprivileged user**, including when the
host command is root. Root invocations create output owned by UID/GID 1000;
otherwise they use the invoking user's UID/GID. Keep the same identity for a
workspace. Use one build per workspace at a time.
Both input checkouts must be readable by that container user; fresh Git clones
have the appropriate permissions.

`setup-packaging.sh` installs Docker, Git and rsync, starts Docker, and builds two cached
images. The build image contains Clang/LLD 18, the repository's Rust toolchain,
Node 22.19.0, Python, WebKit build dependencies and packaging/test tools. The
runtime image reuses `tools/webkit/runtime.Dockerfile` and has no distribution
WebKit/GTK or build SDK installed. Rerun setup after changing either image or
`rust-toolchain.toml`.

The runners use privileged Docker to permit the nested user namespaces needed
by Bubblewrap. WebKit's own process sandbox remains enabled and is checked
before building. This avoids changing the host's global AppArmor settings.
Use these containers for trusted project code on a dedicated build node.

## Build individual stages

```sh
# Tune for available RAM; default is one job per 3 GiB, capped at CPU count.
export BRIMP_BUILD_JOBS=8
bash tools/linux/run-packaging.sh engine /opt/brimp/src/WebKit /opt/brimp/linux
bash tools/linux/run-packaging.sh bindings /opt/brimp/src/WebKit /opt/brimp/linux
bash tools/linux/run-packaging.sh package /opt/brimp/src/WebKit /opt/brimp/linux
bash tools/linux/run-packaging.sh test /opt/brimp/src/WebKit /opt/brimp/linux
```

| Stage | Work |
| --- | --- |
| `doctor` | Check compilers, language runtimes, Python dependencies, disk space and Bubblewrap. |
| `engine` | Call WebKit's `Tools/Scripts/linux-webkit/build.sh`, then its 15 network tests under D-Bus/Xvfb. |
| `bindings` | Download the pinned JSC/curl SDKs, build and repair the Brimp Python wheel, and build/package Node. |
| `package` | Package the built engine SDK and produce stripped, XZ-compressed Python/Node companion packages. |
| `test` | Install both package sets in the minimal runtime container; run automatic discovery and Python/Node backend contracts with the packaged SDK hidden. |
| `all` | Run `engine`, `bindings`, `package`, `test`, stopping on any failure. |

The WebKit recipe remains authoritative for curl-impersonate's pin and engine
feature flags. This directory does not copy those compiler settings. The Brimp
checkout is copied into the workspace before compilation, excluding generated
files and caches. Both input checkouts are mounted read-only. Keep the workspace
outside both checkouts; do not move it after a build because CMake records paths.

Outputs under the work directory:

- `sdk/`: installed WebKit and curl development SDKs.
- `build/`, `cache/`, `src/`: incremental builds, downloads and compiler caches.
- `package/webkit/`: packaged development SDK with provenance and file manifest.
- `dist/base-wheels/`, `dist/base-node/`: Brimp distributions.
- `dist/webkit/`: companion wheel, npm packages and compressed runtime.
- `logs/`: timestamped stage logs, including failed commands; pipeline failures
  propagate through `tee`.

The installed tests mount only the package outputs, packaged SDK and read-only
test source. The original engine install, engine build and native SDK caches
are unavailable. Python and Node extract into separate temporary caches.
This checks actual `engine="webkit"` integration, beyond the engine-only tests.

These are **local development artifacts**. Companion metadata explicitly marks
them as development builds, including when the source checkout is at a tag.
Brimp wheels built here target `manylinux_2_39_x86_64` (Ubuntu 24.04). Production
base Brimp wheels still use the separate `manylinux_2_34` CI environment; use the
release workflows for publishing. Python/Node packages include libatomic;
the minimal test image removes the system copy to verify that bundling.

Validated on the Ubuntu 26.04 node: fresh image setup and sandbox checks, fresh
Python/Node binding builds, 15 engine network tests, and 15 Python plus 2 Node
installed-package contracts. That tooling check reused the existing Linux
WebKit SDK/companion; it did not repeat the full WebKit compilation.
