#!/usr/bin/env python3
"""Check requested Luna access and tool dispatch. This is not a scored run."""

import argparse
import asyncio
import json
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODEL = "gpt-6-luna"
EFFORT = "xhigh"


async def main(binary: str, run_id: str):
    version = subprocess.run([binary, "--version"], check=True, capture_output=True, text=True).stdout.strip()
    output = ROOT / "output" / ("luna-access-" + version.split()[-1]) / run_id
    output.mkdir(parents=True, exist_ok=False)
    process = await asyncio.create_subprocess_exec(
        binary, "app-server", "--listen", "stdio://",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.DEVNULL,
    )
    started = time.monotonic()
    with (output / "events.jsonl").open("w") as events:
        async def send(message):
            process.stdin.write((json.dumps(message) + "\n").encode())
            await process.stdin.drain()

        async def receive():
            raw = await asyncio.wait_for(process.stdout.readline(), 120)
            if not raw:
                raise RuntimeError("app server closed")
            message = json.loads(raw)
            events.write(json.dumps(message) + "\n")
            events.flush()
            return message

        async def request(identifier, method, params):
            await send({"id": identifier, "method": method, "params": params})
            while True:
                message = await receive()
                if message.get("id") == identifier:
                    if "error" in message:
                        raise RuntimeError(message["error"])
                    return message["result"]

        try:
            await request(1, "initialize", {
                "clientInfo": {"name": "choreofix", "title": "ChoreoFix", "version": "0.1.0"},
                "capabilities": {"experimentalApi": True},
            })
            await send({"method": "initialized"})
            thread = await request(2, "thread/start", {
                "model": MODEL,
                "allowProviderModelFallback": False,
                "cwd": str(output),
                "ephemeral": True,
                "environments": [],
                "sandbox": "read-only",
                "approvalPolicy": "never",
                "baseInstructions": "This is a model access check. Use only the supplied probe tool. Do not read files, use other tools, or start agents. Return a brief final confirmation after the probe response.",
                "config": {"model_reasoning_effort": EFFORT, "features.shell_tool": False, "features.multi_agent": False},
                "dynamicTools": [{
                    "type": "function", "name": "probe",
                    "description": "Confirm transport access. Call once with the value 7.",
                    "inputSchema": {"type": "object", "properties": {"value": {"type": "integer"}}, "required": ["value"], "additionalProperties": False},
                }],
            })
            if thread["model"] != MODEL:
                raise RuntimeError(f"requested {MODEL}, returned {thread['model']}")
            await request(3, "turn/start", {
                "threadId": thread["thread"]["id"], "model": MODEL, "effort": EFFORT,
                "input": [{"type": "text", "text": "Call probe with value 7, then confirm access."}],
            })
            calls = 0
            while True:
                message = await receive()
                if message.get("method") == "item/tool/call":
                    params = message["params"]
                    if params["tool"] != "probe" or params["arguments"] != {"value": 7}:
                        raise RuntimeError("unexpected probe tool call")
                    calls += 1
                    await send({"id": message["id"], "result": {"success": True, "contentItems": [{"type": "inputText", "text": "Probe accepted."}]}})
                elif "id" in message and "method" in message:
                    await send({"id": message["id"], "error": {"code": -32601, "message": "Only the probe tool is available."}})
                elif message.get("method") == "turn/completed":
                    turn = message["params"]["turn"]
                    result = {"requested_model": MODEL, "returned_model": thread["model"], "reasoning_effort": EFFORT, "app_server_binary": binary, "app_server_version": version, "status": turn["status"], "probe_calls": calls, "elapsed_seconds": time.monotonic()-started, "error": turn.get("error"), "benchmark_result": False}
                    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
                    print(json.dumps(result))
                    if turn["status"] != "completed" or calls != 1:
                        raise RuntimeError("Luna access/tool check failed")
                    break
        finally:
            process.terminate()
            await process.wait()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app-server-bin", required=True)
    parser.add_argument("--run-id", required=True)
    args = parser.parse_args()
    if not args.run_id or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for c in args.run_id):
        parser.error("run-id must use letters, digits, hyphens, and underscores")
    asyncio.run(main(args.app_server_bin, args.run_id))
