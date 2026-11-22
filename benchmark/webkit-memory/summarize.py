"""Summarize measured process-tree memory; preserve page-load evidence."""

import argparse
import json
import statistics
from collections import defaultdict
from pathlib import Path


def component(command, engine):
    name = Path(command.split()[0]).name if command else "unknown"
    if name == engine:
        return "host"
    if name == "Xvfb":
        return "xvfb"
    for prefix, label in (
        ("WebKitWebProcess", "content"),
        ("WPEWebProcess", "content"),
        ("WebKitNetworkProcess", "network"),
        ("WPENetworkProcess", "network"),
        ("WebKitGPUProcess", "gpu"),
        ("WPEGPUProcess", "gpu"),
        ("dbus", "dbus"),
    ):
        if name.startswith(prefix):
            return label
    return "helpers"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    groups = defaultdict(list)
    failures = []
    for path in sorted(args.directory.glob("*.json")):
        record = json.loads(path.read_text())
        if "engine" not in record:
            continue
        if "median" not in record:
            failures.append({"file": path.name, "error": record.get("error")})
            continue
        groups[(record["engine"], record["workload"])].append(record)
    summary = {"groups": [], "failures": failures}
    for (engine, workload), records in sorted(groups.items()):
        group = {"engine": engine, "workload": workload, "repeats": len(records)}
        for metric in ("pss_kib", "rss_kib", "uss_kib", "xvfb_pss_kib"):
            values = [r["median"][metric] / 1024 for r in records]
            group[metric.replace("_kib", "_mib")] = {
                "median": statistics.median(values),
                "min": min(values),
                "max": max(values),
            }
        group["sampled_peak_pss_mib"] = statistics.median(
            r["sampled_peak_pss_kib"] / 1024 for r in records
        )
        component_trials = defaultdict(list)
        for record in records:
            components = defaultdict(list)
            for sample in record["samples"]:
                if (
                    sample["host_elapsed"] is None
                    or sample["host_elapsed"] < record["steady_from_host_seconds"]
                ):
                    continue
                totals = dict.fromkeys(
                    ("host", "content", "network", "gpu", "dbus", "xvfb", "helpers"), 0
                )
                for process in sample["processes"]:
                    totals[component(process["command"], engine)] += process["pss_kib"] / 1024
                for label, value in totals.items():
                    components[label].append(value)
            for label, values in components.items():
                component_trials[label].append(statistics.median(values))
        group["components_pss_mib"] = {
            label: statistics.median(values) for label, values in component_trials.items()
        }
        group["documents"] = []
        for record in records:
            for event in record["events"]:
                if event.get("event") != "snapshot":
                    continue
                document = event.get("document", {})
                group["documents"].append(
                    {
                        "iteration": record["iteration"],
                        "requested": event["requested"],
                        "status": event["status"],
                        "finished": event["loadFinished"],
                        "navigation_error": event["navigationError"],
                        "evaluation_error": event.get("error"),
                        **document,
                    }
                )
        group["all_documents_observed"] = all(r["all_documents_observed"] for r in records)
        responses = defaultdict(list)
        for record in records:
            snapshots = sorted(
                (e for e in record["events"] if e.get("event") == "snapshot"),
                key=lambda e: e["index"],
            )
            responses[tuple(e["status"] for e in snapshots)].append(record)
        group["response_groups"] = []
        for statuses, matching in responses.items():
            values = [r["median"]["pss_kib"] / 1024 for r in matching]
            group["response_groups"].append(
                {
                    "statuses_in_url_order": statuses,
                    "iterations": [r["iteration"] for r in matching],
                    "pss_mib": {
                        "median": statistics.median(values),
                        "min": min(values),
                        "max": max(values),
                    },
                }
            )
        summary["groups"].append(group)
    (args.directory / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print("engine       workload     repeats   PSS MiB (min–max)        RSS MiB    Xvfb PSS")
    for group in summary["groups"]:
        pss = group["pss_mib"]
        print(
            f"{group['engine']:12} {group['workload']:12} {group['repeats']:>3} "
            f"{pss['median']:9.1f} ({pss['min']:.1f}–{pss['max']:.1f}) "
            f"{group['rss_mib']['median']:12.1f} {group['xvfb_pss_mib']['median']:10.1f}"
        )
    if failures:
        print("Failed trials:", json.dumps(failures))


if __name__ == "__main__":
    main()
