FROM node:22.19.0-bookworm-slim AS node
FROM ubuntu:24.04

ARG RUST_TOOLCHAIN
ENV DEBIAN_FRONTEND=noninteractive
RUN sed -i 's/^Types: deb$/Types: deb deb-src/' /etc/apt/sources.list.d/ubuntu.sources \
    && apt-get update \
    && apt-get build-dep -y --no-install-recommends webkit2gtk \
    && apt-get install -y --no-install-recommends \
        clang-18 lld-18 ccache cmake ninja-build unifdef nasm \
        build-essential libfontconfig1-dev libfreetype6-dev nlohmann-json3-dev \
        curl pkg-config git rsync ca-certificates python3-venv patchelf xz-utils \
        xvfb xauth dbus-x11 bubblewrap xdg-dbus-proxy fonts-dejavu-core \
        libgl1-mesa-dri libegl1 libegl-mesa0 libgles2 libatomic1 libavif16 libjxl0.7 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/npm
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo
ENV PATH=/opt/venv/bin:/opt/cargo/bin:$PATH
RUN test -n "$RUST_TOOLCHAIN" \
    && curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain "$RUST_TOOLCHAIN" \
        --component clippy --component rustfmt --no-modify-path \
    && rm /tmp/rustup.sh \
    && python3 -m venv /opt/venv \
    && python3 -m pip install --no-cache-dir \
        'maturin>=1.9,<2' 'wheel==0.45.1' 'auditwheel==6.8.0' \
        'pytest>=8,<10' 'websockets>=12,<17' 'h2>=4,<5' \
    && userdel ubuntu && useradd -m -u 1000 brimp
USER brimp
