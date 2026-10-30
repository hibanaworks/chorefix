# Experiment runner

The runner records model requests, tool events, versions and usage. Hibana task
permissions belong to the projected Endpoints, not this process interface.
The direct-WASI command currently produces unscored qualification records.
Use the experiment matrix and qualified task artifacts for scored comparisons.

The CI/CD record auditor runs without starting a model or guest:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-cicd EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-cicd-mutants EVENTS.jsonl NEW_DIRECTORY
```

It checks the actual resource loans, requests, returned results, notifications
and original simulator score. Mutation checks remove whole frames or alter
every observation of the same frame, and include overlapping build-server loans.

The writing auditor checks both section Endpoints, actual requests and replies,
returns, original outcomes and disjoint write overlap for 3/5/7 participants:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-writing EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-writing-mutants EVENTS.jsonl NEW_DIRECTORY
```

The actual-WASI writing checks require `CHOREOFIX_WRITING_WASI_BINARY`,
`CHOREOFIX_WRITING_BUILD_RECEIPT` and `CHOREOFIX_EVIDENCE_ROOT`. The receipt
identifies the binary and unchanged source manifest used to build it.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  writing_boundary:: -- --ignored --test-threads=1 --nocapture
```

## Model connection

Participants use the existing ChatGPT login through Codex App Server. They
request `gpt-6-luna` with `xhigh` and reject a substituted model or effort.
The runner requires ChatGPT authentication and removes API-key environment
variables from the child process. It does not call the paid OpenAI API.

Check the login before building:

```sh
codex login status
cargo +stable build --locked --manifest-path tools/runner/Cargo.toml
```

ChatGPT subscription limits still apply. Raw model events, tool calls and
inclusive token usage are recorded for each participant. Reasoning tokens are
part of output tokens and are not counted twice. Missing usage remains unknown.

The original comparison runtimes use Chat Completions. Their subscription
connection has not been qualified. Scored comparisons must wait until that
connection preserves the original prompts, visible history, schemas, tool-call
order, compression and execution engines. A long-lived Codex conversation must
not silently restore history that an original client removed.

## Comparison client fixtures

The local HTTP fixtures call the five unchanged comparison clients and both
original generators. They check two tool rounds, real context compression and
JSON output in 16 requests. They use response fixtures and make no model calls.
The conversion retains optional arguments, schema strictness, parallel-call
settings, call IDs and result order. These tests qualify data conversion only:

```sh
PYTHONHASHSEED=0 PYTHONDONTWRITEBYTECODE=1 \
  cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  original_sync_and_async_comparison_clients_replay_tool_rounds \
  -- --ignored --test-threads=1
PYTHONHASHSEED=0 PYTHONDONTWRITEBYTECODE=1 \
  cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  original_pipeline_clients_keep_compressed_history_and_json_output \
  -- --ignored --test-threads=1
```

Builds and verification must run under Hibana's resource guard with one Cargo
job and the existing 3 GB stop. PyO3 loads the unchanged external comparison
clients and their OpenAI SDK; no Python adapter or evaluated snippet is used.

See [Codex authentication](https://developers.openai.com/codex/auth) and
[App Server](https://developers.openai.com/codex/app-server) for the login and
participant connection.
