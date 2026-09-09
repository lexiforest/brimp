# Benchmarks

The deterministic Rust suite is the development and CI regression gate:

```sh
cargo run -p brimp-lite-worker --example benchmark --release -- --samples 20
```

It uses an in-process resource fixture and reports startup, navigation,
JavaScript, layout, screenshot, throughput, failure rate, staged RSS, repeated
page cleanup, correctness, raw samples, median/p95/p99 latency, environment,
Brimp revision, and fixture revision as one JSON document.

Web-platform conformance is reported separately from performance. After the
pinned sibling WPT runner has produced `wptreport.json`, render the comparable
passing-case headline with:

```sh
python benchmark/wpt_summary.py ../brimp-wpt/results/wptreport.json \
  --output benchmark/wpt-results.md
```

The headline counts passing WPT subtests only. Harness-document success is
reported independently and is not added to the passing-case number.

Generate an actionable failure-cluster report from a completed run with:

```sh
python benchmark/wpt_triage.py ../brimp-wpt/results/wptreport.json \
  --output benchmark/wpt-results/triage.md
```
