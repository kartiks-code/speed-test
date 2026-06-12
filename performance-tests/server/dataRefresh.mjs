/**
 * dataRefresh.mjs — Regenerate viewer/public/data and broadcast SSE updates.
 */

import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PERF_DIR = path.resolve(__dirname, "..");
const BUILD_DATA = path.join(PERF_DIR, "viewer", "scripts", "build-data.mjs");

let broadcastFn = null;

/** Wire SSE broadcast from queue.mjs (avoids circular import at load time). */
export function setDataRefreshBroadcast(fn) {
  broadcastFn = fn;
}

export function regenerateViewerData({ onLog } = {}) {
  const log = onLog ?? (() => {});
  log("[server] Regenerating viewer data...");
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [BUILD_DATA], {
      cwd: PERF_DIR,
      stdio: ["ignore", "pipe", "pipe"],
    });
    child.stdout.on("data", (d) => {
      const line = d.toString().trim();
      if (line) log(line);
    });
    child.stderr.on("data", (d) => {
      const line = d.toString().trim();
      if (line) log(line);
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) reject(new Error(`build-data exited ${code}`));
      else resolve();
    });
  });
}

/** Regenerate viewer data and emit data_updated SSE (runId optional). */
export async function refreshViewerAndBroadcast(runId = null) {
  await regenerateViewerData();
  broadcastFn?.("data_updated", { runId });
}
