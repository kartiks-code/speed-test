/**
 * runs.mjs — Manage completed benchmark runs on disk (suite labels, deletion).
 */

import { readFileSync, writeFileSync, existsSync, readdirSync, rmSync, mkdirSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { execSync } from "child_process";
import { refreshViewerAndBroadcast } from "./dataRefresh.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PERF_DIR = path.resolve(__dirname, "..");
const RESULTS_DIR = path.join(PERF_DIR, "results");

function runDirForId(runId) {
  if (!runId || typeof runId !== "string" || runId.includes("/") || runId.includes("..")) {
    return null;
  }
  const dir = path.join(RESULTS_DIR, runId);
  if (!dir.startsWith(RESULTS_DIR)) return null;
  return dir;
}

function readRunMeta(runId) {
  const dir = runDirForId(runId);
  if (!dir || !existsSync(dir)) return null;
  const metaPath = path.join(dir, "run-meta.json");
  if (!existsSync(metaPath)) return null;
  try {
    return JSON.parse(readFileSync(metaPath, "utf8"));
  } catch {
    return null;
  }
}

function writeRunMeta(runId, meta) {
  const dir = runDirForId(runId);
  if (!dir) throw new Error(`Invalid run id: ${runId}`);
  writeFileSync(path.join(dir, "run-meta.json"), JSON.stringify(meta, null, 2));
}

const ARCHIVE_DIR = path.join(RESULTS_DIR, "archive");

/**
 * Zip a run directory into results/archive/<runId>.zip, then remove the original.
 * Falls back to plain deletion if zip is unavailable.
 */
function archiveAndRemove(runId, dir) {
  mkdirSync(ARCHIVE_DIR, { recursive: true });
  const zipPath = path.join(ARCHIVE_DIR, `${runId}.zip`);
  try {
    execSync(`zip -r ${JSON.stringify(zipPath)} ${JSON.stringify(runId)}`, {
      cwd: RESULTS_DIR,
      stdio: "pipe",
    });
  } catch (err) {
    console.error(`[runs] zip failed for ${runId}: ${err.message}; falling back to plain delete`);
  }
  rmSync(dir, { recursive: true, force: true });
}

function listRunIdsInSuite(suiteName) {
  if (!existsSync(RESULTS_DIR)) return [];
  const ids = [];
  for (const name of readdirSync(RESULTS_DIR)) {
    const meta = readRunMeta(name);
    if (meta?.suite === suiteName) ids.push(name);
  }
  return ids;
}

function normalizeRunIds(runIds) {
  if (!Array.isArray(runIds) || !runIds.length) {
    throw new Error("runIds must be a non-empty array");
  }
  const unique = [...new Set(runIds.map(String))];
  const missing = unique.filter((id) => !readRunMeta(id));
  if (missing.length) {
    throw new Error(`Run(s) not found: ${missing.join(", ")}`);
  }
  return unique;
}

/** Assign a suite name to one or more runs (updates run-meta.json). */
export async function assignSuite({ runIds, suiteName }) {
  const name = typeof suiteName === "string" ? suiteName.trim() : "";
  if (!name) throw new Error("suiteName is required");
  const ids = normalizeRunIds(runIds);

  for (const runId of ids) {
    const meta = readRunMeta(runId);
    meta.suite = name;
    writeRunMeta(runId, meta);
  }

  await refreshViewerAndBroadcast(null);
  return { suiteName: name, count: ids.length, runIds: ids };
}

/** Delete one or more run result directories (archived to results/archive/ first). */
export async function deleteRuns({ runIds }) {
  const ids = normalizeRunIds(runIds);

  for (const runId of ids) {
    const dir = runDirForId(runId);
    archiveAndRemove(runId, dir);
  }

  await refreshViewerAndBroadcast(null);
  return { deleted: ids.length, runIds: ids };
}

/** Remove suite label from all runs in a suite (runs remain on disk). */
export async function dissolveSuite(suiteName) {
  const name = typeof suiteName === "string" ? suiteName.trim() : "";
  if (!name) throw new Error("suite name is required");

  const ids = listRunIdsInSuite(name);
  if (!ids.length) throw new Error(`Suite not found: ${name}`);

  for (const runId of ids) {
    const meta = readRunMeta(runId);
    delete meta.suite;
    writeRunMeta(runId, meta);
  }

  await refreshViewerAndBroadcast(null);
  return { suiteName: name, count: ids.length, runIds: ids };
}

/** Delete all run directories belonging to a suite (archived to results/archive/ first). */
export async function deleteSuiteRuns(suiteName) {
  const name = typeof suiteName === "string" ? suiteName.trim() : "";
  if (!name) throw new Error("suite name is required");

  const ids = listRunIdsInSuite(name);
  if (!ids.length) throw new Error(`Suite not found: ${name}`);

  for (const runId of ids) {
    const dir = runDirForId(runId);
    archiveAndRemove(runId, dir);
  }

  await refreshViewerAndBroadcast(null);
  return { suiteName: name, deleted: ids.length, runIds: ids };
}
