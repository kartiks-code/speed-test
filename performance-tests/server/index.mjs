/**
 * index.mjs — Perf-test control server.
 *
 * Binds on 127.0.0.1:5179. Exposes:
 *   GET  /api/stacks       — stack list with discovered variants
 *   GET  /api/queue        — current job queue
 *   POST /api/queue        — enqueue a new run
 *   DELETE /api/queue/:id  — cancel a pending job
 *   GET  /api/events       — SSE stream (queue updates + live log lines)
 */

import express from "express";
import { readFileSync, readdirSync, existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import {
  getQueue, enqueue, cancelJob,
  addSseClient, removeSseClient,
} from "./queue.mjs";

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
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS");
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
  const { stackId, variant, durationSec, vus, mix, dockerfileOverride } = req.body ?? {};
  if (!stackId || !variant) {
    return res.status(400).json({ error: "stackId and variant are required" });
  }
  const job = enqueue({ stackId, variant, durationSec, vus, mix, dockerfileOverride });
  res.status(201).json(job);
});

app.delete("/api/queue/:id", (req, res) => {
  const ok = cancelJob(req.params.id);
  if (!ok) return res.status(404).json({ error: "Job not found or not cancellable" });
  res.json({ ok: true });
});

// ── /api/events (SSE) ─────────────────────────────────────────────────────────

app.get("/api/events", (req, res) => {
  res.setHeader("Content-Type", "text/event-stream");
  res.setHeader("Cache-Control", "no-cache");
  res.setHeader("Connection", "keep-alive");
  res.flushHeaders();

  // Send initial queue snapshot
  res.write(`data: ${JSON.stringify({ type: "queue_update", queue: getQueue() })}\n\n`);

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
