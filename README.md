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

Source inspection and experiment setup are in progress. No experiment results
have been measured yet.

## Sources

- [TraceFix paper](https://arxiv.org/abs/2605.07935)
- [TraceFix implementation](https://github.com/Sensing-And-Reasoning/TraceFix)
- [Hibana](https://github.com/hibanaworks/hibana)

TraceFix is licensed under MIT. Any retained source keeps its original license.
