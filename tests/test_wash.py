"""Execute shell syntax inside the pinned Hibana WASI engine and ChoreoFS."""

import asyncio
import importlib

import pytest

from benchmark.tools import load_tools
from src.imp.direct_wasi import DirectWasi
from test_cicd_wasi import AGENTS, ROOT


@pytest.mark.asyncio
async def test_scripts_choose_work_and_errors_allow_reconsideration(tmp_path):
    module = importlib.import_module("benchmark.environments.16E.sim")
    sim = module.SimpleCICDSim()
    sim._delay_multiplier = 0.01
    runtime = await asyncio.wait_for(DirectWasi.start(
        ROOT / "target/debug/cicd-wasi",
        ROOT / "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
        load_tools("16E", sim=sim), AGENTS, tmp_path / "runtime",
    ), 10)
    try:
        result = await runtime.command("DEVELOPER", r"""
value='{"component":"application"}'
printf '%s\n' "$value" > /work/0
for word in alpha beta gamma; do
    printf '%s\n' "$word" >> /work/2
done
if cat /work/2 | grep beta; then echo found; else echo wrong; fi
cat < /work/2 | wc
false && echo wrong
cat /missing || echo recovered
n=0
while [ "$n" -lt 3 ]; do
    printf '%s\n' "$n"
    expr "$n" + 1 | read n
done
""")
        assert "beta\nfound\n17\n" in result["stdout"]
        assert "recovered\n0\n1\n2\n" in result["stdout"]
        assert "wrong" not in result["stdout"]
        assert not sim.events
        # Private names are chosen by the script, not a fixed 0..7 catalog.
        await runtime.command("DEVELOPER", "echo notes > /work/agent-notes.txt")
        await runtime.command("DEVELOPER", "echo approved > /work/grant")
        listing = await runtime.command("DEVELOPER", "ls /work")
        assert "agent-notes.txt\n" in listing["stdout"]
        assert "grant\n" in listing["stdout"]
        # A 51,200-byte pipeline cannot fit alongside std, its heap and stack
        # in the initial 64KiB page. It needs real, choreographed memory growth.
        script = "pad='" + "x" * 512 + """'; n=0
while [ "$n" -lt 100 ]; do
    printf '%s' "$pad"
    expr "$n" + 1 | read n
done | wc"""
        grown = await runtime.command("DEVELOPER", script)
        assert "51200\n" in grown["stdout"]
        assert not sim.events
        # A nonterminating computation reaches the shell budget and returns an
        # ordinary error. It does not fault or rebuild a task Endpoint.
        bounded = await runtime.command("DEVELOPER", "while true; do :; done")
        assert "budget exhausted" in bounded["stdout"]
        assert "still-running" in (await runtime.command("DEVELOPER", "echo still-running"))["stdout"]

        # Validate the entire script before producing any file or shared effect.
        rejected = await runtime.command("DEVELOPER", """
cat /work/0 > /service/compile_and_test
if true; then echo unfinished
""")
        assert "wash:" in rejected["stdout"]
        assert not sim.events
        # Persist and source an agent-chosen script, using the normal file ABI.
        await runtime.command("DEVELOPER", r"printf '%s\n' 'echo from-script' > /work/1")
        sourced = await runtime.command("DEVELOPER", ". /work/1")
        assert "from-script\n" in sourced["stdout"]
        # Copy bytes, calculate, or use a direct redirect. The shell has no
        # knowledge of these task operations or their required ordering.
        await runtime.command("DEVELOPER", "cp /work/0 /service/compile_and_test")
        await runtime.command("TESTER", "echo '{\"suite\":\"integration\"}' > /service/run_integration_tests")
        await runtime.command("DEPLOYER", "printf '%s' '{\"target\":\"production\"}' > /service/deploy_application")
        assert sim.is_complete()
        assert not sim.violations
        assert len(sim.events) == 3
        assert await asyncio.gather(*(runtime.command(agent, "exit 0") for agent in AGENTS)) == [{"exit_status": 0}] * 3
        assert (await runtime.result)["status"] == "complete"
        await runtime.reader
    finally:
        await runtime.stop()
