#!/usr/bin/env python3
"""Generate complete TLC-checked baseline protocols using Luna xhigh."""

import argparse
import asyncio
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from src.imp.luna import run_participant, tool_spec
from tracefix.pipeline.prompts import SYSTEM_PROMPT
from tracefix.pipeline.tools import TOOL_REGISTRY, TOOL_SCHEMAS, load_benchmark
from tracefix.pipeline.workspace import Workspace
from tools.check_design import check, configuration_errors


async def design(task_id: str, attempt: str):
    manifest = json.loads((ROOT / "benchmark/upstream-manifest.json").read_text())
    if task_id not in {task["id"] for task in manifest["tasks"]}:
        raise ValueError("unknown benchmark task")
    workspace = Workspace(session_id=task_id, base_dir=str(ROOT / "output" / "baseline-design" / attempt))
    load_benchmark(workspace, task_id=task_id)
    checklist_path = ROOT / "vendor/tracefix/benchmark/environments" / task_id / "checklist.json"
    checklist = checklist_path.read_text()
    lock = asyncio.Lock()

    async def dispatch(name, arguments):
        function = TOOL_REGISTRY[name]
        async with lock:
            if name in {"verify_spec", "extract_states"}:
                errors = configuration_errors(workspace.root)
                if errors:
                    return {"result": "ERROR: " + "; ".join(errors)}
            result = await asyncio.to_thread(function, workspace, **arguments)
            return {"result": result}

    prompt = SYSTEM_PROMPT + f"""

You are designing the complete original benchmark task {task_id}.
The initial task.md, tools.json, metadata.json are already in the workspace.
Read all three with read_file before designing. Preserve every role, resource,
communication dependency, required concurrency, domain operation, and failure
or revision path. Do not simplify a task to make TLC pass. Do not remove roles,
channels, domain work, retries, or checkpoints. Model loops in their full
protocol meaning instead of choosing only a successful path. Preserve domain
work annotations so mediator agents can execute all real benchmark tools.
Run verify_spec and repair genuine protocol errors. Call extract_states only
after verification succeeds. Write notes/fidelity.md explaining each original
requirement below with a specific protocol element. Unsupported requirements
must be reported as missing, not treated as covered.

The checklist's deadlock_scenario entries describe hazards the coordination
protocol must PREVENT. Do not require that a deadlock remain reachable. Follow
the original goal of completing the task without conflicts. Repair the hazard
while preserving task work, roles, resources, individual resource acquisition,
required partial orders and independent work. If a checklist assertion is
mathematically inconsistent, explain it explicitly instead of inventing work.
TLC deadlock detection must remain enabled. Do not weaken invariants or add
state/action constraints to hide counterexamples. Do not disable a check to
obtain PASS. For task 9M/9H, also check eventual completion under the stated fair
scheduling assumptions. Keep all retry/revision branches of other tasks.

Write notes/fidelity.json as an array with exactly one entry per checklist ID:
{{"id":"R1", "status":"covered", "explanation":"...",
  "evidence":[{{"path":"Protocol.tla", "needle":"exact source substring"}}]}}
Allowed statuses: covered, missing, inconsistent. Describe missing/inconsistent
requirements honestly. This evidence is checked separately after your turn.

Original checklist:
{checklist}

When done, leave ir.json, Protocol.tla, Protocol.cfg, states.json, and
notes/fidelity.md in the workspace. Do not call external model APIs or use
native shell tools. Only the supplied design tools are available.
"""
    outcome = await run_participant(
        participant=f"designer-{task_id}", prompt=prompt,
        tools=[tool_spec(schema) for schema in TOOL_SCHEMAS], dispatch=dispatch,
        output=workspace.root / "model-events", timeout=1800, tool_limit=200,
    )
    result = {"task": task_id, "model_outcome": outcome,
              "verification": workspace.result.to_dict(),
              "workspace": str(workspace.root)}
    (workspace.root / "design-result.json").write_text(json.dumps(result, indent=2)+"\n")
    audit = check(workspace.root)
    (workspace.root / "artifact-checks.json").write_text(json.dumps(audit, indent=2)+"\n")
    print(json.dumps({"task": task_id, "model_status": outcome["status"],
                      "tlc_passed": workspace.result.final_passed,
                      "states_present": (workspace.root / "states.json").exists(),
                      "artifact_checks_passed": audit["artifact_checks_passed"]}), flush=True)
    return result


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tasks", nargs="+", required=True)
    parser.add_argument("--attempt", required=True)
    args = parser.parse_args()
    semaphore = asyncio.Semaphore(2)

    async def bounded(task):
        async with semaphore:
            return await design(task, args.attempt)

    await asyncio.gather(*(bounded(task) for task in args.tasks))


if __name__ == "__main__":
    asyncio.run(main())
