"""Terminal RPC for an actual WASI guest. No task progress is tracked here."""

import asyncio
import json
import time


class Shell:
    def __init__(self, process, log, stderr):
        self.process = process
        self.log = log
        self.stderr = stderr
        self.lock = asyncio.Lock()
        self.started = time.monotonic()

    @classmethod
    async def start(cls, binary, guest, output):
        output.mkdir(parents=True, exist_ok=False)
        stderr = (output / "stderr.log").open("wb")
        try:
            process = await asyncio.create_subprocess_exec(
                str(binary), str(guest), stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE, stderr=stderr,
            )
        except BaseException:
            stderr.close()
            raise
        shell = cls(process, (output / "events.jsonl").open("w"), stderr)
        try:
            initial = await shell.receive()
            if initial["type"] != "terminal":
                raise RuntimeError("WASI guest did not open its terminal")
            return shell
        except BaseException:
            await shell.stop()
            raise

    def record(self, direction, message):
        self.log.write(json.dumps({"elapsed_seconds": time.monotonic() - self.started,
                                   "direction": direction, "message": message}) + "\n")
        self.log.flush()

    async def send(self, value):
        self.record("send", value)
        self.process.stdin.write((json.dumps(value) + "\n").encode())
        await self.process.stdin.drain()

    async def receive(self):
        raw = await asyncio.wait_for(self.process.stdout.readline(), 600)
        if not raw:
            raise RuntimeError("WASI shell ended before its terminal reply")
        value = json.loads(raw)
        self.record("receive", value)
        return value

    async def command(self, command, dispatch):
        # One terminal byte stream serializes input, like a normal shell. This
        # lock does not decide which task operation is allowed.
        async with self.lock:
            await self.send({"command": command})
            while True:
                value = await self.receive()
                match value["type"]:
                    case "effect":
                        request = value["request"]
                        response = await dispatch(request)
                        await self.send(response)
                    case "terminal":
                        return {"stdout": value["stdout"]}
                    case "exit":
                        self.process.stdin.close()
                        await asyncio.wait_for(self.process.wait(), 5)
                        return {"exit_status": value["status"]}
                    case _:
                        raise RuntimeError("unknown WASI shell RPC event")

    async def finish(self):
        if self.process.returncode is None:
            result = await self.command("exit", _unexpected_effect)
            if result != {"exit_status": 0}:
                raise RuntimeError("WASI shell did not exit successfully")
        self.process.stdin.close()
        await asyncio.wait_for(self.process.wait(), 5)
        if self.process.returncode != 0:
            raise RuntimeError("WASI shell process failed")

    async def stop(self):
        if not self.process.stdin.is_closing():
            self.process.stdin.close()
        if self.process.returncode is None:
            try:
                await asyncio.wait_for(self.process.wait(), 2)
            except TimeoutError:
                self.process.terminate()
                await self.process.wait()
        self.log.close()
        self.stderr.close()


async def _unexpected_effect(request):
    raise RuntimeError(f"shell exit unexpectedly requested a domain effect: {request}")
