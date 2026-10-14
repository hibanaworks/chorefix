# Experiment specification

## Question

Can Hibana enforce the full TraceFix coordination procedures without a second
runtime state machine, while preserving task behavior and independent work?

## Fixed inputs

- TraceFix revision: `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`.
- Task set: every task from 1E through 16H, with no removed roles or tools.
- Participant model: `gpt-6-luna`.
- Participant reasoning effort: `xhigh`.
- Hibana: published version `0.9.9`, matching main at
  `f1c4dd4bab211b005687cfec15bb53a1f77500ed`.
- Access-checked model harness: desktop bundled CLI `0.162.0-alpha.17.2`.
- Failure conditions: deterministic S0, S1, S2 from the upstream simulator.
- Repetitions: three per task and condition.
- Same domain tool schemas, simulator, checklist, and failure schedule in both arms.

Baseline conditions are awaiting selection. A two-arm comparison has 864 runs.
The paper's four baseline conditions plus Hibana have 2,160 runs for one model.
Model transport, prompts, timeout, and tool-call limits must be fixed before the
first scored run. Access smoke checks are not benchmark results.

The public repository contains task inputs for all 48 cases but translated
protocol fixtures for only 11 cases. These fixtures are not a substitute for a
complete experiment corpus. Protocol generation and fidelity checks must be
completed for all 48 cases before the scored matrix starts.

## Fidelity gate

Each port must account for every original checklist item, participant, resource,
dependency, failure branch, and required concurrency. Compilation alone is not
evidence of fidelity. A reduced workflow, dummy domain operation, or serial
schedule cannot replace the task. Unprojectable cases remain explicit blockers.

Resource access must be represented as protocol communication with affine
ownership. A mutable owner table must not duplicate that ownership in the Hibana
condition. Domain effects happen only after the corresponding endpoint step
permits them. The upstream simulator can independently detect resource misuse.

## Measurements

Record full and partial task completion, simulator violations, protocol
rejections, unconsumed messages and unreleased resources, stalled runs, elapsed
time, model usage including reasoning tokens, and tool-call counts. Save raw
events, configuration, and source hashes with each run.

An out-of-order call rejected before execution is different from a resource
violation executed in the environment. A model stopping early is incomplete
work. A transport or harness error is not a task-success verdict.

## Reporting

Report counts and denominators for all attempted, completed, and failed runs.
Separate access/setup failures from scored runs. Use paired comparisons with
the same task and failure condition. State limits of the simulator and protocol
model. Do not infer domain correctness from communication conformance.
