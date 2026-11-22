# Ubuntu 26.04 development node

The default setup runs Brimp and WebKit builds **natively on Ubuntu 26.04
x86_64**, matching our development node. Docker is not required. For the
separate Ubuntu 24.04 packaging environment, see [PACKAGING.md](PACKAGING.md).

## Set up a new node

As root on a new Ubuntu 26.04 node:

```sh
apt-get update
apt-get install -y git
mkdir -p /opt/brimp/src
git clone https://github.com/lexiforest/brimp.git /opt/brimp/src/brimp
git clone --depth 1 --branch brimp https://github.com/lexiforest/brimp-webkit.git /opt/brimp/src/WebKit
cd /opt/brimp/src/brimp
bash tools/linux/setup.sh brimp
chown -R brimp:brimp /opt/brimp/src/brimp /opt/brimp/src/WebKit
install -d -o brimp -g brimp /opt/brimp/development
runuser -u brimp -- bash tools/linux/run.sh doctor /opt/brimp/src/WebKit /opt/brimp/development
runuser -u brimp -- bash tools/linux/run.sh all /opt/brimp/src/WebKit /opt/brimp/development
```

`setup.sh [USER]` defaults to a `brimp` development account and creates it if
needed. It enables Ubuntu source repositories, installs WebKit's build
requirements, Clang/LLD 18, Node/npm, Xvfb and D-Bus, then installs the
repository's pinned Rust toolchain and a Python test environment for that user.
It verifies Bubblewrap as the normal user. If necessary, it loads the distro's
scoped Bubblewrap AppArmor policy persistently, preserving the global
user-namespace restriction and WebKit's process sandbox.

Builds and tests must run as that development user. When using an existing
account, give its name to setup and make only the chosen source/work directories
writable by it. The scripts never pull or switch either source checkout.

Allow at least 16 GiB RAM and 60 GiB free disk; 32–64 GiB RAM is preferable for
WebKit. The default job count is one per 3 GiB of RAM, capped at CPU count.
Set `BRIMP_BUILD_JOBS` to override it.

## Incremental development

From the Brimp checkout as the development user:

```sh
export BRIMP_BUILD_JOBS=8
bash tools/linux/run.sh engine ../WebKit /opt/brimp/development
bash tools/linux/run.sh bindings ../WebKit /opt/brimp/development
bash tools/linux/run.sh host ../WebKit /opt/brimp/development
bash tools/linux/run.sh test ../WebKit /opt/brimp/development
```

| Stage | Work |
| --- | --- |
| `doctor` | Verify toolchains, Python dependencies, Bubblewrap and available disk. |
| `engine` | Call WebKit's own build recipe, then its network regression suite. |
| `bindings` | Download pinned JSC/curl SDKs and build Python/Node extensions in this checkout. |
| `host` | Compile the Brimp embedding host against the local WebKit SDK. |
| `test` | Run Python/Node backend contracts with D-Bus and Xvfb. |
| `all` | Run `engine`, `bindings`, `host`, `test`, stopping on failure. |

The engine recipe and its feature flags remain in the WebKit repository under
`Tools/Scripts/linux-webkit/`. Brimp's native SDKs go in `.native/development`;
its compiled extensions and embedding host go in the usual checkout locations.
Engine builds, installed SDKs, ccache and timestamped logs stay in the work
directory. Do not move that directory after configuring CMake. Reuse it for
incremental builds, and run only one build per checkout/workspace at a time.

After a Brimp edit, rerun `bindings` and `test`. After an embedding-host edit,
rerun `host` and `test`. After an engine edit, rerun `engine`, `host` and `test`.
A successful WebKit network suite alone does not validate Brimp's
`engine="webkit"` integration.

These native outputs are for development on Ubuntu 26.04. They are not labeled
as portable release packages. Keep a separate work directory for optional
Ubuntu 24.04 packaging checks; the release workflows remain authoritative for
publishing.

Validated natively on Ubuntu 26.04: setup and sandbox checks, fresh Brimp binding
and host builds, 15 Python and 2 Node backend tests, and 15 engine network tests.
The validation reused the existing native WebKit SDK; it did not repeat the
full engine compilation.
