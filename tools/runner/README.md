# Experiment runner

The runner records model requests, tool events, versions and usage. Hibana task
permissions belong to the projected Endpoints, not this process interface.
The direct-WASI command currently produces unscored qualification records.
Use the experiment matrix and qualified task artifacts for scored comparisons.

The trace file and original simulator each have one I/O owner. Callers send
records or simulator requests through channels. A record returns after its
write and flush complete, including any I/O error. The simulator starts the
original asynchronous tools concurrently and returns each actual result on its
request's reply channel.

Source-qualified boundary tests and subscription preflights require a selected
runtime and its build receipt. Set `CHOREOFIX_<FAMILY>_WASI_BINARY` and
`CHOREOFIX_<FAMILY>_BUILD_RECEIPT`, where `FAMILY` is `CODEBASE`, `WRITING`, `API`,
`CICD`, `REVIEW` or `PHILOSOPHERS`, as applicable to the test. A receipt contains
`binary_sha256`, `source_manifest`, `source_manifest_sha256` and
`source_unchanged: true`. Resolve the input manifest relative to the receipt.
The build must compare that source manifest before and after compilation.

Each evidence bundle saves the checked manifest, original receipt and a receipt
whose manifest path is relative to the bundle. Missing or mismatched files stop
the qualification before starting a guest. Existing build archives cannot be
overwritten. These records identify the measured build; they do not establish
that its implementation or task outcome is correct.

The completed code-review auditor checks the repository Endpoint, actual WASI
arguments, simulator requests and results, approval before merging, and guest
exits. Each review uses distinct intent deliveries, a request, an argument-file
close and five result deliveries. Repeated arguments cannot reuse earlier
occurrences. A repeated review also requires a new completed implementation.
Wire labels and lanes are currently checked for consistency between observations
of the same frame, but are not yet matched against a compiled projection artifact.
Consistent changes to those fields can therefore pass this auditor. This limitation
concerns the record audit; it does not disable Hibana's Endpoint checks.
The auditor reports overlapping domain operations and runs without models:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-review EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-review-mutants EVENTS.jsonl NEW_DIRECTORY
```

The review regression requires `CHOREOFIX_REVIEW_WASI_BINARY`,
`CHOREOFIX_REVIEW_BUILD_RECEIPT` and `CHOREOFIX_EVIDENCE_ROOT`. The receipt binds
the binary to its unchanged build-source manifest. Each run saves that manifest,
the receipt, runtime and harness hashes, traces and mutation verdicts under the
evidence root. The workflow runs both ignored review tests and retains these files.
The saved receipt names the manifest relative to its evidence bundle, so moving
the bundle preserves the reference. The original input receipt is saved with its hash.
It covers failure conditions 0/1/2, both first
review targets and repeated or alternating reviews. Eight additional cases use
the original delays and require observed overlap, for 26 cases in total.
Eight separate mechanism cases inject third and fourth rejections through the
unchanged simulator's failure API, covering both first targets and both ways
of revisiting rejected work. They do not change the scored 0/1/2 matrix.
Every trace checks 22 record mutations, plus three that remove a later
same-argument request, argument-file close or result when reviews repeat.
These are integration checks, separate from scored experiments.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  review_boundary:: -- --ignored --test-threads=1 --nocapture
```

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

`thread/inject_items` can supply role-bearing messages and function-call history
to a fresh App Server thread. The converter retains message roles, exact text,
call IDs, argument strings and result order. The HTTP fixtures also save this
input as `app-server-history.json`, including the original clients' trimmed
history. This is input preparation, not a qualified comparison connection.
Output-token limits, tool choice, parallel-call policy, schema strictness and
the boundary of one model response must also be checked before scoring.

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

The installed App Server history RPC can be checked without starting a model
turn. This check requires ChatGPT login and preserves its RPC record under a
new directory in `CHOREOFIX_EVIDENCE_ROOT`:

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  installed_app_server_accepts_role_bearing_history_without_generation \
  -- --ignored --test-threads=1
```

RPC acceptance does not establish which history or generation settings a model
request actually used. Live connection qualification remains required.

Builds and verification must run under Hibana's resource guard with one Cargo
job and the existing 3 GB stop. PyO3 loads the unchanged external comparison
clients and their OpenAI SDK; no Python adapter or evaluated snippet is used.

See [Codex authentication](https://developers.openai.com/codex/auth) and
[App Server](https://developers.openai.com/codex/app-server) for the login and
participant connection.
