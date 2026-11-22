# Deliberately contains no distro WebKit/GTK package or development SDK.
FROM ubuntu:24.04
ARG TEST_UID=1000
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      python3-venv curl ca-certificates xz-utils \
      xvfb xauth dbus-x11 bubblewrap xdg-dbus-proxy fonts-dejavu-core fontconfig libgl1-mesa-dri libegl1 libegl-mesa0 libgles2 \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL https://nodejs.org/dist/v22.19.0/node-v22.19.0-linux-x64.tar.gz -o /tmp/node.tar.gz \
    && tar -xzf /tmp/node.tar.gz -C /usr/local --strip-components=1 \
    && rm /tmp/node.tar.gz \
    && userdel ubuntu && useradd -m -u "$TEST_UID" brimp
# Some graphics dependencies install libatomic transitively. Remove it so a
# passing package test proves the bundled runtime is sufficient.
RUN find /usr/lib -name 'libatomic.so*' -delete
COPY test-runtime.sh /opt/brimp-test-runtime.sh
USER brimp
ENTRYPOINT ["bash", "/opt/brimp-test-runtime.sh"]
