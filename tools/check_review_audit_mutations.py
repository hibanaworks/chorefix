#!/usr/bin/env python3
"""Check the 6E auditor against altered copies of an actual saved model run."""
import argparse
import asyncio
import copy
import hashlib
import json
import shutil
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from tools.audit_review_preflight import audit


async def check(directory):
    control = await audit(directory)
    entries = [json.loads(line) for line in (directory / "io/events.jsonl").read_text().splitlines()]
    result = json.loads((directory / "result.json").read_text())
    outcomes = []
    with tempfile.TemporaryDirectory(prefix="review-evidence-") as scratch:
        target = Path(scratch) / "run"
        shutil.copytree(directory, target)
        for mutation in ("missing_close", "altered_close_source", "forged_feedback", "missing_feedback", "missing_code_ready", "missing_review_channel", "overlapping_resource_intervals"):
            changed = copy.deepcopy(entries)
            frames = [entry["message"] for entry in changed
                      if entry["direction"] == "receive" and entry["message"]["type"] == "protocol_frame"]
            close = next(frame for frame in frames if frame["action"] == "receive"
                         and (frame["source"], frame["target"], frame["payload"]) == (3, 7, [4]))
            if mutation == "missing_close":
                # Remove both observations so frame conservation alone cannot
                # establish that the required close reached the owner.
                changed = [entry for entry in changed if not (
                    entry["message"].get("type") == "protocol_frame"
                    and entry["message"]["frame_id"] == close["frame_id"])]
            elif mutation == "altered_close_source":
                for frame in frames:
                    if frame["frame_id"] == close["frame_id"]:
                        frame["source"] = 1
            elif mutation in ("forged_feedback", "missing_feedback"):
                feedback = next(frame for frame in frames
                                if (frame["source"], frame["target"]) == (3, 0)
                                and bytes(frame["payload"]).startswith(b'{')
                                and "coordination" in json.loads(bytes(frame["payload"])))
                value = json.loads(bytes(feedback["payload"]))
                if mutation == "forged_feedback":
                    verdict = value["coordination"]["code_review"]
                    verdict["success"] = not verdict["success"]
                else:
                    del value["coordination"]
                payload = list(json.dumps(value).encode())
                for frame in frames:
                    if frame["frame_id"] == feedback["frame_id"]:
                        frame["payload"] = payload
            elif mutation in ("missing_code_ready", "missing_review_channel"):
                notification = control["notifications"][0]
                identifier = notification["code_ready_frame" if mutation == "missing_code_ready" else "review_feedback_frame"]
                changed = [entry for entry in changed if not (
                    entry["message"].get("type") == "protocol_frame"
                    and entry["message"]["frame_id"] == identifier)]
            else:
                repo_starts = [entry for entry in changed if entry["direction"] == "domain_start"
                               and entry["message"]["tool"] in {"implement_feature", "merge_code"}]
                first, second = repo_starts[:2]
                delayed = [entry for entry in changed
                           if entry["direction"] in {"domain_end", "send"}
                           and entry["message"].get("id") == first["message"]["id"]
                           and entry["message"].get("type") in {"execute", "simulator_result"}]
                assert len(delayed) == 2
                changed = [entry for entry in changed if entry not in delayed]
                position = changed.index(second) + 1
                changed[position:position] = delayed
            log = target / "io/events.jsonl"
            log.write_text("\n".join(json.dumps(entry) for entry in changed) + "\n")
            changed_result = {**result, "io_events_sha256":hashlib.sha256(log.read_bytes()).hexdigest()}
            (target / "result.json").write_text(json.dumps(changed_result) + "\n")
            try:
                await audit(target)
            except (AssertionError, KeyError, IndexError, ValueError) as error:
                outcomes.append({"mutation":mutation, "rejected":True, "error_type":type(error).__name__})
            else:
                raise AssertionError(f"auditor accepted {mutation}")
    return {"original_audit_passed":control["audit_passed"],
            "run_result_sha256":control["result_sha256"],
            "resigned_io_hashes":True, "mutations":outcomes}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    print(json.dumps(asyncio.run(check(parser.parse_args().directory)), indent=2))
