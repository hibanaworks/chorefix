"""RPC correlation for real guests in one projected task runtime.

This owns terminal streams and simulator call correlation, never task progress
or permission. Only the Rust localsides request simulator effects.
"""

import asyncio
import json
import time


class DirectWasi:
    def __init__(self, process, registry, agents, output, stderr):
        self.process = process
        self.registry = registry
        self.log = (output / "events.jsonl").open("w")
        self.stderr = stderr
        self.started = time.monotonic()
        self.sequence = 0
        self.pending = {}
        self.write_lock = asyncio.Lock()
        self.terminals = {agent: asyncio.Lock() for agent in agents}
        loop = asyncio.get_running_loop()
        self.ready = {agent: loop.create_future() for agent in agents}
        self.result = loop.create_future()
        self.reader = asyncio.create_task(self.read())

    @classmethod
    async def start(cls, binary, guest, registry, agents, output):
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
        runtime = cls(process, registry, agents, output, stderr)
        try:
            await asyncio.gather(*runtime.ready.values())
            return runtime
        except BaseException:
            await runtime.stop()
            raise

    def record(self, direction, message):
        self.log.write(json.dumps({
            "elapsed_seconds": time.monotonic() - self.started,
            "direction": direction, "message": message,
        }) + "\n")
        self.log.flush()

    async def send(self, message):
        async with self.write_lock:
            self.record("send", message)
            self.process.stdin.write((json.dumps(message) + "\n").encode())
            await self.process.stdin.drain()

    async def command(self, agent, command):
        # Serialize bytes on each terminal, without deciding task permissions.
        async with self.terminals[agent]:
            if self.result.done():
                return {"status": "runtime_closed"}
            self.sequence += 1
            identifier = self.sequence
            future = asyncio.get_running_loop().create_future()
            self.pending[identifier] = future
            try:
                await self.send({"type": "command", "id": identifier,
                                 "agent": agent, "command": command})
                return await future
            finally:
                self.pending.pop(identifier, None)

    async def execute(self, message):
        self.record("domain_start", message)
        result = await self.registry.call(
            message["tool"], agent_id=message["agent"], **message["arguments"],
        )
        self.record("domain_end", {**message, "result": result.to_dict()})
        await self.send({"type": "simulator_result", "id": message["id"],
                         "result": result.to_dict()})

    async def read(self):
        executions = set()
        failure = RuntimeError("direct WASI runtime ended before its verdict")
        try:
            while raw := await self.process.stdout.readline():
                message = json.loads(raw)
                self.record("receive", message)
                match message["type"]:
                    case "protocol_frame":
                        pass  # Persisted observation only, never task authority.
                    case "execute":
                        execution = asyncio.create_task(self.execute(message))
                        executions.add(execution)
                        execution.add_done_callback(self.execution_finished)
                    case "terminal" if message["id"] is None:
                        self.ready[message["agent"]].set_result(message)
                    case "terminal" | "exit":
                        future = self.pending[message["id"]]
                        if not future.done():
                            future.set_result(
                                {"stdout": message["stdout"]}
                                if message["type"] == "terminal" else
                                {"exit_status": message["status"], "stdout": message["stdout"]}
                            )
                    case "terminal_closed":
                        future = self.pending[message["id"]]
                        if not future.done():
                            future.set_result({"status": "guest_closed"})
                    case "run_result":
                        failure = RuntimeError(message.get("error", "runtime completed"))
                        if not self.result.done():
                            self.result.set_result(message)
                        self.process.stdin.close()
                        await self.process.stdin.wait_closed()
                        break
                    case _:
                        raise RuntimeError("unknown direct WASI RPC event")
            if not self.result.done():
                self.result.set_exception(failure)
        except Exception as error:
            failure = error
            if not self.result.done():
                self.result.set_exception(error)
        finally:
            if self.result.done() and not self.result.cancelled():
                if (error := self.result.exception()) is not None:
                    failure = error
            for future in [*self.ready.values(), *self.pending.values()]:
                if not future.done():
                    future.set_exception(failure)
            for execution in executions:
                if not execution.done():
                    execution.cancel()
            await asyncio.gather(*executions, return_exceptions=True)

    def execution_finished(self, execution):
        if not execution.cancelled() and (error := execution.exception()) is not None:
            if not self.result.done():
                self.result.set_exception(error)
            self.reader.cancel()

    async def stop(self):
        if not self.process.stdin.is_closing():
            self.process.stdin.close()
        if self.process.returncode is None:
            try:
                await asyncio.wait_for(self.process.wait(), 2)
            except TimeoutError:
                self.process.terminate()
                await self.process.wait()
        if not self.reader.done():
            self.reader.cancel()
        await asyncio.gather(self.reader, return_exceptions=True)
        # Retrieve a failed verdict even if startup or a participant failed first.
        if self.result.done() and not self.result.cancelled():
            self.result.exception()
        elif not self.result.done():
            self.result.cancel()
        self.log.close()
        self.stderr.close()
