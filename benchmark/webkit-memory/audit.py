"""Validate measurement accounting and actual rendered-view evidence."""

import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    plan = json.loads((args.directory / "plan.json").read_text())
    settings = set()
    for engine, workload, iteration in plan["trials"]:
        name = f"{engine}-{workload}-{iteration}"
        record = json.loads((args.directory / f"{name}.json").read_text())
        assert "error" not in record, (name, record.get("error"))
        assert record["all_documents_observed"], name
        started = next(e for e in record["events"] if e.get("event") == "started")
        settings.add(tuple(started[k] for k in ("cacheModel", "defaultFont", "pageCache", "webgl")))
        steady = [
            s
            for s in record["samples"]
            if s["host_elapsed"] is not None
            and s["host_elapsed"] >= record["steady_from_host_seconds"]
        ]
        assert len(steady) >= 8, (name, len(steady))
        for sample in record["samples"]:
            pids = [p["pid"] for p in sample["processes"]]
            assert len(pids) == len(set(pids)), (name, "duplicate PID")
            for key in ("pss_kib", "rss_kib", "uss_kib"):
                assert sample[key] == sum(p[key] for p in sample["processes"]), (name, key)
            assert sample["rss_kib"] >= sample["pss_kib"], name
            assert sample["memory.swap.current"] == 0, (name, "swap used")
            if engine.startswith("wpe"):
                assert sample["xvfb_pss_kib"] == 0, name
                assert not any("Xvfb" in p["command"] for p in sample["processes"]), name
        assert record["median"]["pss_kib"] > 0, name
        if engine.startswith("gtk"):
            assert record["median"]["xvfb_pss_kib"] > 0, name
        snapshots = [e for e in record["events"] if e.get("event") == "snapshot"]
        assert sorted(e["index"] for e in snapshots) == list(range(len(record["urls"]))), name
        for event in snapshots:
            assert "error" not in event, (name, event)
            document = event["document"]
            assert document["viewport"] == [1280, 720, 1], (name, document["viewport"])
            assert document["screen"] == [1280, 720, 24], (name, document["screen"])
            assert document["visibility"] == "visible", (name, document["visibility"])
            assert document["focused"] == (event["index"] == len(record["urls"]) - 1), (
                name,
                "focus differs",
            )
    assert len(settings) == 1, settings
    print(
        f"Validated {len(plan['trials'])} trials: accounting, no swap, library/view evidence and matching runtime settings."
    )
    print(
        "HTTP success and meaningful site content require separate review; this audit does not count block pages as successful loads."
    )


if __name__ == "__main__":
    main()
