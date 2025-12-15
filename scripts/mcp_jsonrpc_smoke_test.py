#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import sys
import threading
from pathlib import Path

CONFIG_PATH = Path('.mcp.json')


def write_lsp_message(stdin, obj):
    body = json.dumps(obj, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
    stdin.write(header)
    stdin.write(body)
    stdin.flush()


def read_lsp_message(stdout):
    # Read headers
    headers = {}
    while True:
        line = stdout.readline()
        if not line:
            return None
        try:
            s = line.decode("utf-8", errors="replace")
        except Exception:
            s = str(line)
        if s in ("\r\n", "\n"):
            break
        if ":" in s:
            k, v = s.split(":", 1)
            headers[k.strip().lower()] = v.strip()
    try:
        length = int(headers.get("content-length", "0"))
    except ValueError:
        length = 0
    if length <= 0:
        return None
    body = stdout.read(length)
    if not body:
        return None
    return json.loads(body.decode("utf-8"))


def load_server_from_config(server_name: str = "memory-mcp"):
    if not CONFIG_PATH.exists():
        raise FileNotFoundError(f"Config not found: {CONFIG_PATH}")
    with CONFIG_PATH.open("r", encoding="utf-8") as f:
        cfg = json.load(f)
    servers = cfg.get("mcpServers") or {}
    if server_name not in servers:
        raise KeyError(f"Server '{server_name}' not found in {CONFIG_PATH}")
    s = servers[server_name]
    cmd = s.get("command")
    args = s.get("args") or []
    env = s.get("env") or {}
    if not cmd:
        raise ValueError(f"No command configured for server '{server_name}'")
    return cmd, args, env


def spawn_server(command, args=None, env=None):
    args = args or []
    env_vars = os.environ.copy()
    if env:
        # Do not overwrite PATH etc. Merge provided env
        env_vars.update({k: str(v) for k, v in env.items()})
    # Ensure logs don't pollute stdout
    env_vars.setdefault("RUST_LOG", "off")

    proc = subprocess.Popen(
        [command] + list(args),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=0,
    )

    def drain_stderr():
        for line in iter(proc.stderr.readline, b""):
            try:
                sys.stderr.write(line.decode("utf-8", errors="replace"))
            except Exception:
                pass
    threading.Thread(target=drain_stderr, daemon=True).start()
    return proc


def run_sequence(proc):
    # Assign fixed IDs so we can correlate responses even if out-of-order
    REQ_INIT = 1
    REQ_LIST = 2
    REQ_HEALTH = 3
    REQ_QUERY = 4
    REQ_SHUTDOWN = 5

    # Send requests except shutdown first
    write_lsp_message(proc.stdin, {"jsonrpc": "2.0", "id": REQ_INIT, "method": "initialize"})
    write_lsp_message(proc.stdin, {"jsonrpc": "2.0", "id": REQ_LIST, "method": "tools/list"})
    write_lsp_message(
        proc.stdin,
        {
            "jsonrpc": "2.0",
            "id": REQ_HEALTH,
            "method": "tools/call",
            "params": {"name": "health_check", "arguments": {}},
        },
    )
    # Add query_memory sample call
    write_lsp_message(
        proc.stdin,
        {
            "jsonrpc": "2.0",
            "id": REQ_QUERY,
            "method": "tools/call",
            "params": {
                "name": "query_memory",
                "arguments": {
                    "query": "implement async storage",
                    "domain": "web-api",
                    "task_type": "code_generation",
                    "limit": 3
                },
            },
        },
    )

    # Collect responses by id (wait up to a bounded number of messages)
    wanted = {REQ_INIT, REQ_LIST, REQ_HEALTH, REQ_QUERY}
    responses = {}

    import time, select
    deadline = time.time() + 15.0  # 15s overall to gather responses
    while wanted and time.time() < deadline:
        # Wait for readability with short timeout
        rlist, _, _ = select.select([proc.stdout], [], [], 0.5)
        if not rlist:
            continue
        resp = read_lsp_message(proc.stdout)
        if resp is None:
            continue
        rid = resp.get("id")
        responses[rid] = resp
        if rid in wanted:
            wanted.remove(rid)

    # Now send shutdown and wait for its response
    write_lsp_message(proc.stdin, {"jsonrpc": "2.0", "id": REQ_SHUTDOWN, "method": "shutdown"})
    shutdown_resp = None
    shutdown_deadline = time.time() + 5.0
    while time.time() < shutdown_deadline:
        rlist, _, _ = select.select([proc.stdout], [], [], 0.5)
        if not rlist:
            continue
        resp = read_lsp_message(proc.stdout)
        if resp and resp.get("id") == REQ_SHUTDOWN:
            shutdown_resp = resp
            break

    # Print in logical order regardless of arrival order
    print("initialize =>", json.dumps(responses.get(REQ_INIT), indent=2))
    print("tools/list =>", json.dumps(responses.get(REQ_LIST), indent=2))
    print("tools/call(health_check) =>", json.dumps(responses.get(REQ_HEALTH), indent=2))
    print("tools/call(query_memory) =>", json.dumps(responses.get(REQ_QUERY), indent=2))
    print("shutdown =>", json.dumps(shutdown_resp, indent=2))


def main():
    parser = argparse.ArgumentParser(description="Smoke test for memory-mcp JSON-RPC stdio server using .mcp.json config")
    parser.add_argument("--server-name", default="memory-mcp", help="Server key from .mcp.json (default: memory-mcp)")
    parser.add_argument("--compat", default="false", help="Enable MCP_COMPAT_ALIASES (true/false)")
    parser.add_argument("--override-cmd", default=None, help="Override command path (optional)")
    args = parser.parse_args()

    cmd, cmd_args, cmd_env = load_server_from_config(args.server_name)
    if args.override_cmd:
        cmd = args.override_cmd

    compat = args.compat.lower() in ("true", "1", "yes")
    if compat:
        cmd_env = dict(cmd_env) if cmd_env else {}
        cmd_env["MCP_COMPAT_ALIASES"] = "true"

    # Spawn server using config
    proc = spawn_server(cmd, args=cmd_args, env=cmd_env)

    try:
        run_sequence(proc)
    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
