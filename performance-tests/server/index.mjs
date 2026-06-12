/**
 * index.mjs — Perf-test control server.
 *
 * Binds on 127.0.0.1:5179. Exposes:
 *   GET  /api/stacks       — stack list with discovered variants
 *   GET  /api/queue        — current job queue
 *   POST /api/queue        — enqueue a new run
 *   DELETE /api/queue/completed — remove finished jobs from the queue UI
 *   DELETE /api/queue/:id  — cancel a pending job
 *   POST /api/runs/assign-suite — assign suite label to existing runs
 *   DELETE /api/runs       — delete completed run directories
 *   DELETE /api/suites/:name — dissolve or delete suite runs
 *   GET  /api/events       — SSE stream (queue updates, live log lines, data refresh)
 */

import express from "express";
import { readFileSync, readdirSync, existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import {
  getQueue, getQueueSummary, enqueue, enqueueBatch, cancelJob, clearCompletedJobs,
  addSseClient, removeSseClient,
} from "./queue.mjs";
import { assignSuite, deleteRuns, dissolveSuite, deleteSuiteRuns } from "./runs.mjs";
import { refreshViewerAndBroadcast } from "./dataRefresh.mjs";

const __dirname  = path.dirname(fileURLToPath(import.meta.url));
const PERF_DIR   = path.resolve(__dirname, "..");
const REPO_ROOT  = path.resolve(PERF_DIR, "..");
const STACKS_JSON = path.join(PERF_DIR, "stacks.json");
const PORT       = parseInt(process.env.CONTROL_PORT || "5179", 10);

const app = express();
app.use(express.json());

// Allow the Vite dev server to call us during local development
app.use((req, res, next) => {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, DELETE, PATCH, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
  if (req.method === "OPTIONS") return res.sendStatus(204);
  next();
});

// ── /api/stacks ───────────────────────────────────────────────────────────────

app.get("/api/stacks", (req, res) => {
  let stacks;
  try {
    stacks = JSON.parse(readFileSync(STACKS_JSON, "utf8"));
  } catch (err) {
    return res.status(500).json({ error: `Cannot read stacks.json: ${err.message}` });
  }

  const result = stacks.map((stack) => {
    const buildCtx = path.join(REPO_ROOT, stack.build_context);
    let files = [];
    try {
      files = readdirSync(buildCtx).filter((f) => f.startsWith("Dockerfile"));
    } catch (_) {}

    const variants = files
      .map((f) => {
        if (f === "Dockerfile") return { label: "naive", dockerfile: "Dockerfile" };
        if (f === "Dockerfile.optimized") return { label: "optimized", dockerfile: "Dockerfile.optimized" };
        // e.g. Dockerfile.alpine → alpine
        const suffix = f.replace(/^Dockerfile\.?/, "");
        return { label: suffix || "default", dockerfile: f };
      })
      .sort((a, b) => a.label.localeCompare(b.label));

    return {
      id: stack.id,
      label: stack.label,
      variants,
      notes: stack.notes ?? null,
    };
  });

  res.json(result);
});

// ── /api/queue ────────────────────────────────────────────────────────────────

app.get("/api/queue", (req, res) => {
  res.json(getQueue());
});

app.post("/api/queue", (req, res) => {
  const body = req.body ?? {};
  const {
    stackId, variant, stackIds, variants, suiteName,
    durationSec, vus, mix, dockerfileOverride,
  } = body;

  const shared = { suiteName, durationSec, vus, mix, dockerfileOverride };

  if (Array.isArray(stackIds) && Array.isArray(variants)) {
    if (!stackIds.length || !variants.length) {
      return res.status(400).json({ error: "stackIds and variants must be non-empty arrays" });
    }
    const name = typeof suiteName === "string" ? suiteName.trim() : "";
    if (!name) {
      return res.status(400).json({ error: "suiteName is required when enqueueing multiple runs" });
    }
    const jobs = enqueueBatch({ suiteName: name, stackIds, variants, ...shared });
    return res.status(201).json({ suiteName: name, count: jobs.length, jobs });
  }

  if (!stackId || !variant) {
    return res.status(400).json({ error: "stackId and variant are required" });
  }
  const job = enqueue({ stackId, variant, ...shared });
  res.status(201).json(job);
});

app.delete("/api/queue/completed", (req, res) => {
  const removed = clearCompletedJobs();
  res.json({ ok: true, removed });
});

app.delete("/api/queue/:id", (req, res) => {
  const ok = cancelJob(req.params.id);
  if (!ok) return res.status(404).json({ error: "Job not found or not cancellable" });
  res.json({ ok: true });
});

// ── /api/runs — manage completed results ─────────────────────────────────────

app.post("/api/runs/assign-suite", async (req, res) => {
  try {
    const { runIds, suiteName } = req.body ?? {};
    const result = await assignSuite({ runIds, suiteName });
    res.json(result);
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

app.delete("/api/runs", async (req, res) => {
  try {
    const { runIds } = req.body ?? {};
    const result = await deleteRuns({ runIds });
    res.json(result);
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

app.delete("/api/suites/:name", async (req, res) => {
  try {
    const suiteName = decodeURIComponent(req.params.name);
    const action = req.query.action === "delete-runs" ? "delete-runs" : "dissolve";
    const result = action === "delete-runs"
      ? await deleteSuiteRuns(suiteName)
      : await dissolveSuite(suiteName);
    res.json({ action, ...result });
  } catch (err) {
    const status = err.message.includes("not found") ? 404 : 400;
    res.status(status).json({ error: err.message });
  }
});

// ── /api/events (SSE) ─────────────────────────────────────────────────────────

app.get("/api/events", (req, res) => {
  res.setHeader("Content-Type", "text/event-stream");
  res.setHeader("Cache-Control", "no-cache");
  res.setHeader("Connection", "keep-alive");
  res.flushHeaders();

  // Send initial queue snapshot
  res.write(`data: ${JSON.stringify({ type: "queue_update", queue: getQueueSummary() })}\n\n`);

  addSseClient(res);

  // Heartbeat to keep the connection alive through proxies
  const hb = setInterval(() => { try { res.write(": heartbeat\n\n"); } catch (_) {} }, 15000);

  req.on("close", () => {
    clearInterval(hb);
    removeSseClient(res);
  });
});

// ── start ─────────────────────────────────────────────────────────────────────

app.listen(PORT, "127.0.0.1", () => {
  console.log(`[server] Perf-test control server listening on http://127.0.0.1:${PORT}`);
  console.log(`[server] CORS enabled for all origins (local-only, no auth)`);
});

// ── Periodic viewer refresh ───────────────────────────────────────────────────
// Re-scans results/ every 15 s. Because build-data.mjs only includes
// directories that exist on disk, any run whose folder has been removed
// (externally or via archive) is automatically dropped from the listing.

let periodicRefreshRunning = false;
setInterval(async () => {
  if (periodicRefreshRunning) return;
  periodicRefreshRunning = true;
  try {
    await refreshViewerAndBroadcast();
  } catch (err) {
    console.error(`[server] Periodic refresh failed: ${err.message}`);
  } finally {
    periodicRefreshRunning = false;
  }
}, 15_000);
