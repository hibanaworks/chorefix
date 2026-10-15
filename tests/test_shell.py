"""Real guest/ChoreoFS/process checks. Build with tools/build_shell.sh first."""
import json
from pathlib import Path

import pytest
from src.imp.shell import Shell

ROOT = Path(__file__).resolve().parents[1]
GUEST = ROOT / "guests/shell/target/wasm32-wasip1/release/tracefix-shell-guest.wasm"


@pytest.mark.asyncio
async def test_choreofs_files_scripts_copy_and_truncate(tmp_path):
    shell = await Shell.start(ROOT / "target/debug/wasi-shell", GUEST, tmp_path / "shell")
    async def forbidden(request):
        pytest.fail(f"private work caused a shared effect: {request}")
    try:
        listing = await shell.command("ls /work", forbidden)
        for number in range(8):
            assert f"{number}\n" in listing["stdout"]
        await shell.command("echo a long old value > /work/0", forbidden)
        await shell.command("echo hi > /work/0", forbidden)
        assert (await shell.command("cat /work/0", forbidden))["stdout"] == "hi\nwasi> "
        await shell.command("apply /work/0 /work/2", forbidden)
        assert (await shell.command("cat /work/2", forbidden))["stdout"] == "hi\nwasi> "
        await shell.command(r"echo cat /work/0\ncat /work/2 > /work/1", forbidden)
        assert (await shell.command("script /work/1", forbidden))["stdout"] == "hi\nhi\nwasi> "
        # Neither a pathname nor a private approval file grants service rights.
        await shell.command("echo approved > /work/7", forbidden)
        assert "denied" in (await shell.command("echo unauthorized > /tools/reply", forbidden))["stdout"]
        assert "denied" in (await shell.command("echo unauthorized > /tmp/native-escape", forbidden))["stdout"]
        await shell.finish()
    finally:
        await shell.stop()


@pytest.mark.asyncio
async def test_script_preserves_actual_tool_arguments_across_chunked_wasi_io(tmp_path):
    shell = await Shell.start(ROOT / "target/debug/wasi-shell", GUEST, tmp_path / "shell")
    observed = []
    request = {"tool": "design_feature", "arguments": {"feature_name": "a" * 220}}
    async def dispatch(value):
        observed.append(value)
        return {"success": True, "data": {"text": "b" * 250}}
    try:
        await shell.command("echo call " + json.dumps(request) + " > /work/0", dispatch)
        assert not observed
        response = await shell.command("script /work/0", dispatch)
        assert observed == [request]
        assert '"success":true' in response["stdout"]
        assert "b" * 250 in response["stdout"]
        await shell.finish()
    finally:
        await shell.stop()


@pytest.mark.asyncio
async def test_wasi_choreofs_cannot_bypass_the_task_endpoint(tmp_path):
    import asyncio
    shell = await Shell.start(ROOT / "target/debug/wasi-shell", GUEST, tmp_path / "shell")
    task = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/philosophers"), "9E",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
    )
    observed = []
    async def dispatch(request):
        task.stdin.write((json.dumps({"type":"call", "id":1, "agent":"PHILOSOPHER_A", **request}) + "\n").encode())
        await task.stdin.drain()
        reply = json.loads(await asyncio.wait_for(task.stdout.readline(), 5))
        observed.append(reply)
        # If an execute request appears, the actual effect boundary was crossed.
        assert reply["type"] == "run_result"
        assert reply["status"] == "failed"
        assert "PhaseInvariant" in reply["error"]
        return {"protocol_error":reply["error"]}
    try:
        await shell.command('echo approved > /work/7', dispatch)
        result = await shell.command('call {"tool":"eat","arguments":{"philosopher":"PHILOSOPHER_A"}}', dispatch)
        assert "protocol_error" in result["stdout"]
        assert len(observed) == 1
        await shell.finish()
    finally:
        task.stdin.close()
        await asyncio.wait_for(task.wait(), 5)
        await shell.stop()
