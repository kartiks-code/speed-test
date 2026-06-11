#!/usr/bin/env python3
"""
sampler.py — Poll Docker Engine API every 1s and write CPU/RAM/network/block-IO
stats for specified container names to a CSV file.

Usage:
    python sampler.py --containers <name1> [<name2> ...] --output <path/to/docker-stats.csv>

The script runs until it receives SIGTERM or SIGINT (i.e. run.sh kills it after the
k6 run completes). It uses the Docker Unix socket directly via urllib so no
third-party packages are required.

CPU calculation follows the Docker convention:
    cpu_delta = cpu_stats.cpu_usage.total_usage - precpu_stats.cpu_usage.total_usage
    system_delta = cpu_stats.system_cpu_usage - precpu_stats.system_cpu_usage
    cpu_percent = (cpu_delta / system_delta) * num_cpus * 100.0
"""

import argparse
import csv
import http.client
import json
import signal
import socket
import sys
import time
from datetime import datetime, timezone


# ── Docker socket HTTP client ────────────────────────────────────────────────

class UnixSocketHTTPConnection(http.client.HTTPConnection):
    def __init__(self, socket_path: str):
        super().__init__("localhost")
        self._socket_path = socket_path

    def connect(self):
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(self._socket_path)
        self.sock = sock


def docker_get(path: str, socket_path: str = "/var/run/docker.sock") -> dict:
    conn = UnixSocketHTTPConnection(socket_path)
    conn.request("GET", path)
    resp = conn.getresponse()
    data = resp.read()
    conn.close()
    if resp.status != 200:
        raise RuntimeError(f"Docker API {path} returned {resp.status}: {data[:200]}")
    return json.loads(data)


# ── Stats parsing ────────────────────────────────────────────────────────────

def parse_stats(raw: dict, container_name: str, ts: str) -> dict:
    """Extract relevant metrics from a single /containers/{id}/stats?stream=false response."""
    row = {
        "timestamp": ts,
        "container": container_name,
        "cpu_percent": 0.0,
        "mem_usage_mb": 0.0,
        "mem_limit_mb": 0.0,
        "mem_percent": 0.0,
        "net_rx_mb": 0.0,
        "net_tx_mb": 0.0,
        "blk_read_mb": 0.0,
        "blk_write_mb": 0.0,
    }

    # CPU
    cpu = raw.get("cpu_stats", {})
    precpu = raw.get("precpu_stats", {})
    cpu_usage = cpu.get("cpu_usage", {}).get("total_usage", 0)
    precpu_usage = precpu.get("cpu_usage", {}).get("total_usage", 0)
    sys_usage = cpu.get("system_cpu_usage", 0)
    presys_usage = precpu.get("system_cpu_usage", 0)
    num_cpus = cpu.get("online_cpus") or len(cpu.get("cpu_usage", {}).get("percpu_usage") or [1])
    cpu_delta = cpu_usage - precpu_usage
    sys_delta = sys_usage - presys_usage
    if sys_delta > 0 and cpu_delta > 0:
        row["cpu_percent"] = round((cpu_delta / sys_delta) * num_cpus * 100.0, 3)

    # Memory
    mem = raw.get("memory_stats", {})
    # Use working_set (usage minus cache) when available; fall back to raw usage
    usage = mem.get("stats", {}).get("active_anon", None)
    if usage is None:
        usage = mem.get("usage", 0) - mem.get("stats", {}).get("cache", 0)
    limit = mem.get("limit", 0)
    row["mem_usage_mb"] = round(usage / 1_048_576, 3)
    row["mem_limit_mb"] = round(limit / 1_048_576, 3)
    if limit > 0:
        row["mem_percent"] = round(usage / limit * 100.0, 3)

    # Network (sum all interfaces)
    networks = raw.get("networks", {})
    rx_bytes = sum(v.get("rx_bytes", 0) for v in networks.values())
    tx_bytes = sum(v.get("tx_bytes", 0) for v in networks.values())
    row["net_rx_mb"] = round(rx_bytes / 1_048_576, 6)
    row["net_tx_mb"] = round(tx_bytes / 1_048_576, 6)

    # Block I/O
    blkio = raw.get("blkio_stats", {})
    io_service = blkio.get("io_service_bytes_recursive") or []
    for entry in io_service:
        op = entry.get("op", "").lower()
        val = entry.get("value", 0)
        if op == "read":
            row["blk_read_mb"] += val
        elif op == "write":
            row["blk_write_mb"] += val
    row["blk_read_mb"] = round(row["blk_read_mb"] / 1_048_576, 6)
    row["blk_write_mb"] = round(row["blk_write_mb"] / 1_048_576, 6)

    return row


FIELDNAMES = [
    "timestamp", "container",
    "cpu_percent",
    "mem_usage_mb", "mem_limit_mb", "mem_percent",
    "net_rx_mb", "net_tx_mb",
    "blk_read_mb", "blk_write_mb",
]


# ── Main loop ────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Sample Docker container stats to CSV")
    parser.add_argument("--containers", nargs="+", required=True, help="Container names to monitor")
    parser.add_argument("--output", required=True, help="Output CSV path")
    parser.add_argument("--interval", type=float, default=1.0, help="Sampling interval in seconds (default: 1)")
    parser.add_argument("--socket", default="/var/run/docker.sock", help="Docker socket path")
    args = parser.parse_args()

    stop = False

    def _handle_signal(signum, frame):
        nonlocal stop
        stop = True

    signal.signal(signal.SIGTERM, _handle_signal)
    signal.signal(signal.SIGINT, _handle_signal)

    with open(args.output, "w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=FIELDNAMES)
        writer.writeheader()
        fh.flush()

        print(f"[sampler] Writing to {args.output}, sampling {args.containers} every {args.interval}s", flush=True)

        while not stop:
            ts = datetime.now(timezone.utc).isoformat()
            for name in args.containers:
                try:
                    raw = docker_get(f"/containers/{name}/stats?stream=false", args.socket)
                    row = parse_stats(raw, name, ts)
                    writer.writerow(row)
                except Exception as exc:
                    print(f"[sampler] WARN: could not sample {name}: {exc}", file=sys.stderr, flush=True)
            fh.flush()
            time.sleep(args.interval)

    print("[sampler] Done.", flush=True)


if __name__ == "__main__":
    main()
