"""Interactive variants of the pyv8c examples: pointer travel, then a click.

The plain batch loads each target and waits, which is what `probe.py` measures.
The examples that need a gesture are different: Akamai scores the pointer, and the
Cloudflare "5s" flow needs a click on a checkbox rendered inside a cross-origin
frame. This script drives `page.mouse` for both and reports what the site did.
"""

from __future__ import annotations

import argparse
import json
import math
import time

import brimp


def cookies(session):
    return {record["name"]: record["value"] for record in session.cookies}


def travel(page, from_point, to_point, samples=24):
    """Move in eased steps, as a hand does, rather than one jump."""
    (x0, y0), (x1, y1) = from_point, to_point
    for index in range(1, samples + 1):
        ratio = index / samples
        eased = 0.5 - 0.5 * math.cos(math.pi * ratio)
        page.mouse.move(x0 + (x1 - x0) * eased, y0 + (y1 - y0) * eased)


def pump(page, seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        page.run_pending_tasks()
        time.sleep(0.25)


def akamai(url, seconds):
    """pyv8c/examples/akamai: pointer travel, a click, then `_abck` with `~0~`."""
    result = {"target": "akamai", "url": url}
    with brimp.Session() as session:
        page = session.get(url, timeout=60)
        result["status"] = page.status_code
        width, height = page.evaluate("[innerWidth, innerHeight]") or [800, 600]
        point = [width * 0.2, height * 0.3]
        for seed in (0.35, 0.6, 0.8):
            target = [width * seed, height * (0.25 + 0.35 * seed)]
            travel(page, point, target)
            page.mouse.move(target[0], target[1])
            pump(page, 0.6)
            point = target
        page.mouse.click(point[0], point[1], delay=0.07)
        pump(page, max(5.0, seconds))
        jar = cookies(session)
        result["_abck"] = jar.get("_abck", "")[:48]
        result["pass"] = "~0~" in jar.get("_abck", "")
        result["cookies"] = sorted(jar)
    return result


def cloudflare(url, seconds):
    """pyv8c/examples/5s: click the Turnstile checkbox, then `cf_clearance`."""
    result = {"target": "cloudflare/5s", "url": url, "attempts": []}
    with brimp.Session() as session:
        page = session.get(url, timeout=60)
        result["status"] = page.status_code
        frames = page.evaluate(
            "Array.from(document.querySelectorAll('iframe'), (frame) => {"
            "  const box = frame.getBoundingClientRect();"
            "  return [frame.src || '(no src)', Math.round(box.x), Math.round(box.y),"
            "          Math.round(box.width), Math.round(box.height)]; })"
        )
        result["frames"] = frames
        # The widget places its checkbox near the left edge, vertically centred; try
        # a few offsets because the exact position depends on the widget's layout.
        for frame in frames or []:
            (src, x, y, width, height) = frame
            if width < 24 or height < 24:
                continue
            for fx, fy in ((0.12, 0.5), (0.08, 0.5), (0.2, 0.5), (0.5, 0.5), (0.12, 0.35)):
                target = [x + width * fx, y + height * fy]
                travel(page, [0, 0], target, samples=12)
                page.mouse.click(target[0], target[1], delay=0.08)
                pump(page, 1.5)
                jar = cookies(session)
                entry = {
                    "frame": src[:60],
                    "point": [round(v) for v in target],
                    "cookies": sorted(jar),
                }
                result["attempts"].append(entry)
                if "cf_clearance" in jar:
                    result["pass"] = True
                    result["cf_clearance"] = jar["cf_clearance"][:32]
                    result["cookies"] = sorted(jar)
                    return result
        pump(page, max(3.0, seconds))
        jar = cookies(session)
        result["pass"] = "cf_clearance" in jar
        result["cookies"] = sorted(jar)
        result["title"] = page.evaluate("document.title")
    return result


def px3(url, seconds):
    """pyv8c/examples/px3: press and hold the challenge button."""
    result = {"target": "px3", "url": url}
    with brimp.Session() as session:
        page = session.get(url, timeout=60)
        result["status"] = page.status_code
        box = page.evaluate(
            "(() => { const node = document.querySelector('#px-captcha');"
            "  if (!node) return null; const box = node.getBoundingClientRect();"
            "  return [Math.round(box.x), Math.round(box.y), Math.round(box.width), Math.round(box.height),"
            "          (node.textContent || '').replace(/\\s+/g, ' ').trim().slice(0, 40)]; })()"
        )
        result["widget"] = box
        if box and box[2] > 8:
            point = [box[0] + box[2] / 2, box[1] + box[3] / 2]
            travel(page, [0, 0], point, samples=16)
            page.mouse.down()
            pump(page, 3.0)
            page.mouse.up()
            pump(page, max(2.0, seconds))
        jar = cookies(session)
        result["cookies"] = sorted(jar)
        result["pass"] = any(name.startswith("_px") and value for name, value in jar.items())
        result["url_after"] = page.url
    return result


TARGETS = {
    "akamai": (akamai, "https://www.radissonhotels.com/zh-cn/"),
    "cloudflare": (cloudflare, "https://steamdb.info/login/"),
    "px3": (px3, "https://www.skyscanner.com.au/sttc/px/captcha-v2/index.html"),
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("target", nargs="*", default=list(TARGETS))
    parser.add_argument("--seconds", type=float, default=15)
    arguments = parser.parse_args()
    for name in arguments.target:
        function, url = TARGETS[name]
        started = time.monotonic()
        result = function(url, arguments.seconds)
        result["elapsed"] = round(time.monotonic() - started, 1)
        print(json.dumps(result, indent=1)[:2500])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
