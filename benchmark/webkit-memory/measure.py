"""Root supervisor: run sandboxed browsers as brimp, measure isolated cgroups."""

import argparse
import json
import os
import pwd
import random
import selectors
import shutil
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

SITES = {
    "wikipedia": "https://en.wikipedia.org/wiki/Main_Page",
    "youtube": "https://www.youtube.com/",
    "reddit": "https://www.reddit.com/r/popular/",
    "amazon": "https://www.amazon.com/",
    "bbc": "https://www.bbc.com/news",
}
ENGINES = {
    "gtk-soup": ("webkit-baseline", "gtk"),
    "gtk-curl": ("webkit-curl", "gtk"),
    "wpe-soup": ("wpe-memory", "wpe"),
}


def memory(group):
    records = []
    for file in group.rglob("cgroup.procs"):
        for entry in file.read_text().split():
            proc = Path("/proc") / entry
            try:
                values = {}
                for line in (proc / "smaps_rollup").read_text().splitlines()[1:]:
                    key, value, *_ = line.split()
                    values[key.rstrip(":")] = int(value)
                command = (
                    (proc / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace")
                )
                records.append(
                    {
                        "pid": int(entry),
                        "command": command,
                        "pss_kib": values["Pss"],
                        "rss_kib": values["Rss"],
                        "uss_kib": sum(
                            values.get(key, 0)
                            for key in ("Private_Clean", "Private_Dirty", "Private_Hugetlb")
                        ),
                    }
                )
            except (FileNotFoundError, ProcessLookupError):
                continue
    result = {
        key: sum(record[key] for record in records) for key in ("pss_kib", "rss_kib", "uss_kib")
    }
    result["xvfb_pss_kib"] = sum(
        record["pss_kib"]
        for record in records
        if record["command"] and Path(record["command"].split()[0]).name == "Xvfb"
    )
    result["processes"] = records
    for name in ("memory.current", "memory.peak", "memory.swap.current"):
        result[name] = int((group / name).read_text())
    return result


def run(engine, workload, iteration, args):
    urls = (
        list(SITES.values())
        if workload == "all-five"
        else ["about:blank" if workload == "blank" else SITES[workload]]
    )
    sdk_name, port = ENGINES[engine]
    sdk = Path("/opt/brimp/sdk") / sdk_name
    viewer = args.bin / engine
    unit = f"brimp-memory-{os.getpid()}-{engine}-{workload}-{iteration}.service"
    group = Path("/sys/fs/cgroup/system.slice") / unit
    identity = pwd.getpwnam("brimp")
    temporary = Path(tempfile.mkdtemp(prefix="brimp-memory-"))
    os.chown(temporary, identity.pw_uid, identity.pw_gid)
    environment = {
        "LD_LIBRARY_PATH": str(sdk / "lib"),
        "GTK_A11Y": "none",
        "LIBGL_ALWAYS_SOFTWARE": "1",
        "WPE_PLATFORM": "headless",
        "LC_ALL": "C.UTF-8",
        "XDG_RUNTIME_DIR": str(temporary),
    }
    for name in ("XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME"):
        directory = temporary / name.lower()
        directory.mkdir()
        os.chown(directory, identity.pw_uid, identity.pw_gid)
        environment[name] = str(directory)
    command = [
        "systemd-run",
        "--quiet",
        "--pipe",
        "--wait",
        "--collect",
        f"--unit={unit}",
        "-p",
        "User=brimp",
        "-p",
        "MemoryAccounting=yes",
        "-p",
        "KillMode=control-group",
        "-p",
        "TimeoutStopSec=5s",
        "-p",
        "RuntimeMaxSec=120s",
    ]
    for key, value in environment.items():
        command += ["-p", f"Environment={key}={value}"]
    command += ["--", "dbus-run-session", "--"]
    if port == "gtk":
        command += ["xvfb-run", "-a", "--server-args=-screen 0 1280x720x24 -nolisten tcp"]
    command += [str(viewer), *urls]
    name = f"{engine}-{workload}-{iteration}"
    record = {
        "engine": engine,
        "workload": workload,
        "iteration": iteration,
        "urls": urls,
        "command": command,
        "events": [],
        "samples": [],
    }
    started = None
    launched = time.monotonic()
    snapshots = 0
    buffer = b""
    steady_at = args.settle if workload != "blank" else 8
    try:
        with (args.output / f"{name}.stderr").open("wb") as errors:
            process = subprocess.Popen(
                command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors
            )
            selector = selectors.DefaultSelector()
            selector.register(process.stdout, selectors.EVENT_READ)
            next_sample = launched
            while True:
                now = time.monotonic()
                if now - launched > 90:
                    raise TimeoutError("Browser did not complete the measurement window")
                for key, _ in selector.select(timeout=0.05):
                    data = os.read(key.fd, 65536)
                    if not data:
                        raise RuntimeError(f"Browser exited early: {process.poll()}")
                    buffer += data
                    while b"\n" in buffer:
                        line, buffer = buffer.split(b"\n", 1)
                        try:
                            event = json.loads(line)
                        except ValueError:
                            continue
                        event["elapsed"] = time.monotonic() - launched
                        record["events"].append(event)
                        if event.get("event") == "started":
                            started = time.monotonic()
                            expected = (sdk / "lib").resolve()
                            if not Path(event["library"]).resolve().is_relative_to(expected):
                                raise RuntimeError(f"Unexpected engine library: {event}")
                if group.exists() and now >= next_sample:
                    sample = memory(group)
                    sample["elapsed"] = now - launched
                    sample["host_elapsed"] = now - started if started else None
                    record["samples"].append(sample)
                    next_sample = now + 0.5
                if started:
                    elapsed = now - started
                    if elapsed >= steady_at - 5 and snapshots == 0:
                        process.stdin.write(b"s")
                        process.stdin.flush()
                        snapshots += 1
                    if elapsed >= steady_at + args.window:
                        break
            selector.close()
            record["steady_from_host_seconds"] = steady_at
            record["steady_window_seconds"] = args.window
            steady = [
                s
                for s in record["samples"]
                if s["host_elapsed"] is not None and s["host_elapsed"] >= steady_at
            ]
            if not steady:
                raise RuntimeError("No steady-window memory samples")
            record["median"] = {
                key: statistics.median(s[key] for s in steady)
                for key in ("pss_kib", "rss_kib", "uss_kib", "xvfb_pss_kib")
            }
            record["sampled_peak_pss_kib"] = max(s["pss_kib"] for s in record["samples"])
            record["cgroup_peak_bytes"] = max(s["memory.peak"] for s in record["samples"])
            documents = [e for e in record["events"] if e.get("event") == "snapshot"]
            record["all_documents_observed"] = len(documents) == len(urls)
    except (OSError, RuntimeError, ValueError) as error:
        record["error"] = repr(error)
    finally:
        subprocess.run(["systemctl", "stop", unit], capture_output=True, check=False)
        if "process" in locals():
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        shutil.rmtree(temporary)
        (args.output / f"{name}.json").write_text(json.dumps(record, indent=2) + "\n")
    summary = {
        key: record.get(key)
        for key in ("engine", "workload", "iteration", "median", "all_documents_observed", "error")
    }
    print(json.dumps(summary), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engines", nargs="+", choices=ENGINES, default=["gtk-soup", "wpe-soup"])
    parser.add_argument(
        "--workloads",
        nargs="+",
        choices=["blank", *SITES, "all-five"],
        default=["blank", *SITES, "all-five"],
    )
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--settle", type=float, default=20)
    parser.add_argument("--window", type=float, default=5)
    parser.add_argument("--bin", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    trials = [
        (engine, workload, iteration)
        for iteration in range(args.repeats)
        for workload in args.workloads
        for engine in args.engines
    ]
    random.Random(20261120).shuffle(trials)
    (args.output / "plan.json").write_text(
        json.dumps(
            {"sites": SITES, "trials": trials, "arguments": vars(args)}, default=str, indent=2
        )
        + "\n"
    )
    for trial in trials:
        run(*trial, args)


if __name__ == "__main__":
    main()
