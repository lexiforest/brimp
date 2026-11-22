# GTK/Xvfb and WPE headless memory experiment

This experiment compares real WebKit rendering on Linux using the same small
GLib embedding consumer, compiled for GTK and WPE. It measures the engine and
platform processes, not the Python/Node bindings or Brimp's Inspector adapter.
It does not replace the working GTK Brimp host.

The requested live sites are Wikipedia's main page, YouTube, Reddit's popular
feed, Amazon and BBC News. Each site is tested separately, then all five are
opened in one host with a shared ephemeral network session. An `about:blank`
case measures the base platform cost. All views are visible at 1280×720 CSS
pixels with device scale 1; this is not a foreground/background browser-tab
benchmark. No account login, consent interaction or video playback is performed.

## Controls

- `gtk-soup`: existing baseline WebKitGTK SDK with Soup networking and Xvfb.
- `wpe-soup`: WPE built from the same Safari-branch source baseline with its
  native headless platform and Soup networking. No X server or Wayland compositor.
- `gtk-curl`: the existing curl-impersonate GTK SDK, to show whether the actual
  current network build changes the result.

The primary platform comparison is `gtk-soup` versus `wpe-soup`. Both use
Release builds, JavaScriptCore, Skia, WebGL, software graphics and the same
desktop user agent. Video, Web Audio, WebRTC, gamepads and speech are disabled,
matching the existing GTK test SDK. In particular, this does not measure
YouTube video playback or a fully featured multimedia distribution. Platform
defaults such as pointer lock, touch automation and Web API statistics differ;
the effective `cmakeconfig.h` files are retained alongside the results.

## Measurement

The root supervisor starts each browser as the unprivileged `brimp` user in a
separate transient systemd service. WebKit sandboxing stays enabled. Each trial
gets fresh XDG directories, an ephemeral cookie store and a private D-Bus
session. GTK gets one private Xvfb server for the whole trial. The cgroup includes
the viewer, content/network/GPU processes, sandbox helpers, Xvfb and D-Bus
services. The Python measurement supervisor is outside it.

Memory is sampled every 0.5 seconds using `/proc/PID/smaps_rollup` for all
processes in the cgroup. The main metric is summed PSS, which apportions shared
pages rather than counting them once per process. RSS, private resident memory,
Xvfb's own PSS, sampled peak PSS and cgroup memory charges are also retained.
Cgroup charges include additional kernel/cache costs and are not interchangeable
with PSS. See the Linux [proc](https://docs.kernel.org/filesystems/proc.html) and
[cgroup](https://docs.kernel.org/admin-guide/cgroup-v2.html) documentation.

For live pages, the reported value is the median sample from 20–25 seconds after
host initialization; blank pages use 8–13 seconds. Each case runs three times
in a reproducibly shuffled order, with only one browser trial active at once.
The summary reports the median of trial medians and their min/max range. These
are fixed-time observations, not proof that dynamic sites have reached idle.

At 15 seconds (3 seconds for blank), the viewer waits for two animation frames
and records the final URL, HTTP status, title, ready state, viewport, visibility,
DOM/image counts and text excerpt. Block pages and incomplete shells must be
identified when interpreting results, even when HTTP status is 200. Live-site
content, ads, experiments and regional behavior can vary between trials.

## Reproduce on the experiment machine

Install the engine build dependencies described in WebKit's
`Tools/Scripts/linux-webkit/README.md`, plus `nlohmann-json3-dev`. Build an isolated
WPE SDK without modifying the GTK SDK:

```sh
patch -d /opt/brimp/src/WebKit-memory -p1 < benchmark/webkit-memory/wpe-build.patch
bash benchmark/webkit-memory/build-wpe.sh /opt/brimp/src/WebKit-memory /opt/brimp
mkdir -p /opt/brimp/experiments/webkit-memory/bin
bash benchmark/webkit-memory/build-viewer.sh /opt/brimp/sdk/webkit-baseline gtk /opt/brimp/experiments/webkit-memory/bin/gtk-soup
bash benchmark/webkit-memory/build-viewer.sh /opt/brimp/sdk/webkit-curl gtk /opt/brimp/experiments/webkit-memory/bin/gtk-curl
bash benchmark/webkit-memory/build-viewer.sh /opt/brimp/sdk/wpe-memory wpe /opt/brimp/experiments/webkit-memory/bin/wpe-soup
sudo python3 benchmark/webkit-memory/measure.py \
  --engines gtk-soup wpe-soup gtk-curl \
  --bin /opt/brimp/experiments/webkit-memory/bin \
  --output /opt/brimp/experiments/webkit-memory/results
python3 benchmark/webkit-memory/summarize.py /opt/brimp/experiments/webkit-memory/results
python3 benchmark/webkit-memory/audit.py /opt/brimp/experiments/webkit-memory/results
```

The runner uses the SDK paths above and the existing `brimp` test user. Its
systemd services have a 120-second lifetime limit and are stopped after every
trial, including failures. Raw JSON and stderr are preserved per trial.
The audit checks process accounting, absence of swap, matching runtime settings,
viewport, screen, visibility/focus, and the absence of Xvfb from WPE trials.
It deliberately does not treat successful instrumentation as proof that a site
served its real content; HTTP statuses and text evidence need separate review.
