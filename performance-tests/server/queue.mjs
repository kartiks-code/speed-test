/**
 * queue.mjs — Sequential job queue manager for the perf-test control server.
 *
 * Jobs are processed one at a time; each job spawns run.sh with the caller's
 * parameters, streams stdout/stderr to all connected SSE clients, then
 * regenerates viewer/public/data by running build-data.mjs.
 */

import { spawn } from "child_process";
import { readFileSync, readdirSync, existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { createRequire } from "module";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PERF_DIR  = path.resolve(__dirname, "..");
const REPO_ROOT = path.resolve(PERF_DIR, "..");
const RUN_SH    = path.join(PERF_DIR, "run.sh");
const BUILD_DATA = path.join(PERF_DIR, "viewer", "scripts", "build-data.mjs");
const RESULTS_DIR = path.join(PERF_DIR, "results");

// ── SSE broadcast ─────────────────────────────────────────────────────────────

const sseClients = new Set();

export function addSseClient(res) {
  sseClients.add(res);
}

export function removeSseClient(res) {
  sseClients.delete(res);
}

function broadcast(type, data) {
  const msg = `data: ${JSON.stringify({ type, ...data })}\n\n`;
  for (const res of sseClients) {
    try { res.write(msg); } catch (_) {}
  }
}

// ── queue state ───────────────────────────────────────────────────────────────

let jobIdCounter = 1;
const queue = [];   // { id, stackId, variant, durationSec, vus, mix, status, runId, createdAt, startedAt, finishedAt }
let running = false;

export function getQueue() {
  return queue.map(j => ({ ...j }));
}

export function enqueue(params) {
  const job = {
    id: String(jobIdCounter++),
    stackId: params.stackId,
    variant: params.variant,
    durationSec: params.durationSec ?? 60,
    vus: params.vus ?? 20,
    mix: {
      create: params.mix?.create ?? 25,
      read:   params.mix?.read   ?? 25,
      update: params.mix?.update ?? 25,
      delete: params.mix?.delete ?? 25,
    },
    dockerfileOverride: params.dockerfileOverride ?? "",
    status: "pending",
    runId: null,
    createdAt: new Date().toISOString(),
    startedAt: null,
    finishedAt: null,
    log: [],
  };
  queue.push(job);
  broadcast("queue_update", { queue: getQueue() });
  setImmediate(processNext);
  return job;
}

export function cancelJob(id) {
  const idx = queue.findIndex(j => j.id === id);
  if (idx === -1) return false;
  const job = queue[idx];
  if (job.status !== "pending") return false;
  job.status = "canceled";
  broadcast("queue_update", { queue: getQueue() });
  return true;
}

// ── processor ─────────────────────────────────────────────────────────────────

async function processNext() {
  if (running) return;
  const job = queue.find(j => j.status === "pending");
  if (!job) return;

  running = true;
  job.status = "running";
  job.startedAt = new Date().toISOString();
  broadcast("queue_update", { queue: getQueue() });

  try {
    await runJob(job);
    job.status = "done";
  } catch (err) {
    job.status = "failed";
    appendLog(job, `[server] ERROR: ${err.message}`);
  }

  job.finishedAt = new Date().toISOString();
  broadcast("queue_update", { queue: getQueue() });

  // Regenerate viewer data for the new result
  try {
    await regenerateViewerData(job);
  } catch (err) {
    appendLog(job, `[server] build-data warn: ${err.message}`);
  }

  running = false;
  setImmediate(processNext);
}

function appendLog(job, line) {
  job.log.push(line);
  broadcast("log", { jobId: job.id, line });
}

function runJob(job) {
  return new Promise((resolve, reject) => {
    const env = {
      ...process.env,
      VUS: String(job.vus),
      DURATION: `${job.durationSec}s`,
      K6_SCRIPT_NAME: "crud-mix.js",
      MIX_CREATE: String(job.mix.create),
      MIX_READ:   String(job.mix.read),
      MIX_UPDATE: String(job.mix.update),
      MIX_DELETE: String(job.mix.delete),
    };
    if (job.dockerfileOverride) {
      env.DOCKERFILE_OVERRIDE = job.dockerfileOverride;
    }

    appendLog(job, `[server] Starting: ${job.stackId} ${job.variant} VUs=${job.vus} Duration=${job.durationSec}s Mix=${JSON.stringify(job.mix)}`);

    const child = spawn("bash", [RUN_SH, job.stackId, job.variant], {
      cwd: PERF_DIR,
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });

    const onLine = (chunk) => {
      const lines = chunk.toString().split("\n");
      for (const line of lines) {
        if (line.trim()) appendLog(job, line);
      }
    };

    child.stdout.on("data", onLine);
    child.stderr.on("data", onLine);

    child.on("error", reject);
    child.on("close", (code) => {
      // Detect the run_id from the newest results directory for this stack+variant
      job.runId = findRunId(job.stackId, job.variant);
      appendLog(job, `[server] run.sh exited with code ${code}. runId=${job.runId ?? "unknown"}`);
      if (code !== 0) {
        reject(new Error(`run.sh exited ${code}`));
      } else {
        resolve();
      }
    });
  });
}

function findRunId(stackId, variant) {
  if (!existsSync(RESULTS_DIR)) return null;
  const dirs = readdirSync(RESULTS_DIR)
    .filter(d => d.startsWith(`${stackId}-${variant}-`))
    .sort()
    .reverse();
  return dirs[0] ?? null;
}

async function regenerateViewerData(job) {
  appendLog(job, "[server] Regenerating viewer data...");
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [BUILD_DATA], {
      cwd: PERF_DIR,
      stdio: ["ignore", "pipe", "pipe"],
    });
    child.stdout.on("data", (d) => appendLog(job, d.toString().trim()));
    child.stderr.on("data", (d) => appendLog(job, d.toString().trim()));
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) reject(new Error(`build-data exited ${code}`));
      else resolve();
    });
  });
}
