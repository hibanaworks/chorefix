# Hibana TraceFix experiment

This repository tests whether Hibana can enforce the communication procedures
in TraceFix with multiple GPT-6 Luna agents using xhigh reasoning.

The task set remains the full 16 scenarios at three difficulty levels, for 48
tasks. The experiment keeps the original task descriptions, tools, simulator,
and coordination checklists. It replaces coordination in the Hibana condition
with global protocols and attached endpoints.

Hibana owns coordination progress. Participant code does not keep another state
machine or flags to decide which communication step is allowed. The external
simulator owns the domain environment and checks task outcomes.

The upstream implementation is pinned to
`36b4bd108fcd74f7efb17613aebbe3fdf55499a9`. Its current runtime checks step order,
which differs from the topology-only monitor described in the paper. Results
will state the baseline revision explicitly.

## Status

All 48 tasks and 825 checklist items have a recorded byte-level source manifest.
Three concurrent GPT-6 Luna access checks completed with xhigh reasoning and
successful tool calls. These checks are recorded in
[the setup report](results/setup-report.json), separately from benchmark results.

The coordination port and scored benchmark runs have not started yet. The
comparison conditions are awaiting selection.

Fetch and check the fixed upstream inputs:

```sh
python3 tools/bootstrap_upstream.py
python3 tools/inventory.py --check
```

Run an access check with an explicitly selected app-server binary:

```sh
python3 tools/probe_luna.py --app-server-bin /path/to/cli --run-id access-1
```

The verified binary is the desktop application's bundled CLI,
`0.162.0-alpha.17.2`. The older PATH CLI `0.145.0` rejected GPT-6 Luna with this
account. The probe fails on a substituted model or unsuccessful turn.

## Sources

- [TraceFix paper](https://arxiv.org/abs/2605.07935)
- [TraceFix implementation](https://github.com/Sensing-And-Reasoning/TraceFix)
- [Hibana](https://github.com/hibanaworks/hibana)

TraceFix is licensed under MIT. Any retained source keeps its original license.
