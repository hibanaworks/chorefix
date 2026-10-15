"""One independent Luna participant, with tool calls delegated to its owner."""

import asyncio
import hashlib
import json
import time
from pathlib import Path
import jsonschema

MODEL = "gpt-6-luna"
EFFORT = "xhigh"
VERIFIED_CLI = "/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex"


def tool_spec(function: dict) -> dict:
    return {
        "type": "function",
        "name": function["name"],
        "description": function["description"],
        "inputSchema": function["parameters"],
    }


async def run_participant(*, participant: str, prompt: str, tools: list[dict],
                          dispatch, output: Path, timeout: float,
                          tool_limit: int, binary: str = VERIFIED_CLI) -> dict:
    """No coordination state lives here. Dispatch owns each tool operation."""
    output.mkdir(parents=True, exist_ok=False)
    process = await asyncio.create_subprocess_exec(
        binary, "app-server", "--listen", "stdio://",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.DEVNULL,
        limit=8*1024*1024,
    )
    started = time.monotonic()
    rpc_lock = asyncio.Lock()
    requests = {}
    calls = set()
    completion = asyncio.get_running_loop().create_future()
    usage = None
    call_count = 0
    thread_model = None
    thread_effort = None
    tool_schemas = {tool["name"]: tool["inputSchema"] for tool in tools}
    with (output / "events.jsonl").open("w") as log:
        def record(direction, message):
            log.write(json.dumps({"elapsed_seconds": time.monotonic()-started, "direction": direction, "message": message}) + "\n")
            log.flush()

        async def send(message):
            async with rpc_lock:
                record("send", message)
                process.stdin.write((json.dumps(message) + "\n").encode())
                await process.stdin.drain()

        async def request(identifier, method, params):
            future = asyncio.get_running_loop().create_future()
            requests[identifier] = future
            await send({"id": identifier, "method": method, "params": params})
            return await future

        async def execute(message):
            nonlocal call_count
            call_count += 1
            if call_count > tool_limit:
                raise RuntimeError("participant tool-call budget exhausted")
            params = message["params"]
            try:
                jsonschema.validate(params["arguments"], tool_schemas[params["tool"]])
            except (jsonschema.ValidationError, KeyError) as error:
                await send({"id":message["id"], "result":{
                    "success":False, "contentItems":[{"type":"inputText", "text":json.dumps({
                        "status":"invalid_tool_arguments", "error":str(error)})}]}})
                return
            outcome = await dispatch(params["tool"], params["arguments"])
            await send({"id": message["id"], "result": {
                "success": True,
                "contentItems": [{"type": "inputText", "text": json.dumps(outcome)}],
            }})

        def tool_finished(task):
            calls.remove(task)
            if task.cancelled():
                return
            error = task.exception()
            if error is not None and not completion.done():
                completion.set_exception(error)

        async def read():
            nonlocal usage
            try:
                while raw := await process.stdout.readline():
                    message = json.loads(raw)
                    record("receive", message)
                    if "id" in message and "method" not in message:
                        future = requests.pop(message["id"])
                        if "error" in message:
                            future.set_exception(RuntimeError(message["error"]))
                        else:
                            future.set_result(message["result"])
                    elif message.get("method") == "item/tool/call":
                        task = asyncio.create_task(execute(message))
                        calls.add(task)
                        task.add_done_callback(tool_finished)
                    elif "id" in message and "method" in message:
                        await send({"id": message["id"], "error": {
                            "code": -32601, "message": "Only the supplied experiment tools are available.",
                        }})
                    elif message.get("method") == "thread/tokenUsage/updated":
                        usage = message["params"]["tokenUsage"]["total"]
                    elif message.get("method") == "turn/completed":
                        if not completion.done():
                            completion.set_result(message["params"]["turn"])
                if not completion.done():
                    raise RuntimeError("participant model process closed before completion")
            except Exception as error:
                for future in requests.values():
                    if not future.done():
                        future.set_exception(error)
                if not completion.done():
                    completion.set_exception(error)

        reader = asyncio.create_task(read())
        try:
            async with asyncio.timeout(timeout):
                await request(1, "initialize", {
                    "clientInfo": {"name": "hibana_tracefix", "title": "Hibana TraceFix", "version": "0.1.0"},
                    "capabilities": {"experimentalApi": True},
                })
                await send({"method": "initialized"})
                thread = await request(2, "thread/start", {
                    "model": MODEL, "allowProviderModelFallback": False,
                    "cwd": str(output.resolve()), "ephemeral": True,
                    "environments": [], "sandbox": "read-only", "approvalPolicy": "never",
                    "baseInstructions": prompt + "\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response.",
                    "config": {"model_reasoning_effort": EFFORT, "features.shell_tool": False,
                               "features.multi_agent": False, "project_doc_max_bytes": 0},
                    "dynamicTools": tools,
                })
                thread_model, thread_effort = thread["model"], thread["reasoningEffort"]
                if thread_model != MODEL or thread_effort != EFFORT:
                    raise RuntimeError("participant model or reasoning effort was substituted")
                await request(3, "turn/start", {
                    "threadId": thread["thread"]["id"], "model": MODEL, "effort": EFFORT,
                    "input": [{"type": "text", "text": "Begin your assigned work. Use the experiment tools to carry it out."}],
                })
                turn = await completion
                result = {"participant": participant, "status": turn["status"], "error": turn.get("error")}
        except TimeoutError:
            result = {"participant": participant, "status": "timeout", "error": "participant timeout"}
        except asyncio.CancelledError:
            result = {"participant": participant, "status": "cancelled", "error": "participant run cancelled"}
        except Exception as error:
            result = {"participant": participant, "status": "harness_error", "error": str(error)}
        finally:
            for task in tuple(calls):
                task.cancel()
            await asyncio.gather(*tuple(calls), return_exceptions=True)
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
            for future in requests.values():
                if not future.done():
                    future.cancel()
                elif not future.cancelled():
                    future.exception()
            if completion.done() and not completion.cancelled():
                completion.exception()
            if process.returncode is None:
                process.terminate()
            try:
                await asyncio.wait_for(process.wait(), timeout=10)
            except TimeoutError:
                process.kill()
                await process.wait()
        result.update({"requested_model": MODEL, "returned_model": thread_model,
                       "requested_effort": EFFORT, "returned_effort": thread_effort,
                       "tool_calls": call_count, "elapsed_seconds": time.monotonic()-started,
                       "usage": usage})
    result["raw_events_sha256"] = hashlib.sha256((output / "events.jsonl").read_bytes()).hexdigest()
    (output / "participant.json").write_text(json.dumps(result, indent=2) + "\n")
    return result
