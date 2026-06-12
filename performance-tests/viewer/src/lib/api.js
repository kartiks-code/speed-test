/**
 * api.js — client helpers for the perf-test control server.
 *
 * All requests go through /api (proxied by Vite to 127.0.0.1:5179 in dev,
 * or served by the control server directly when the built SPA is used).
 */

const BASE = "/api";

async function apiFetch(path, init) {
  const res = await fetch(`${BASE}${path}`, init);
  if (!res.ok) {
    let msg = `${res.status} ${res.statusText}`;
    try { msg = (await res.json()).error ?? msg; } catch (_) {}
    throw new Error(msg);
  }
  return res.json();
}

/**
 * Fetch the list of stacks with their discovered variants.
 * Returns: Array<{ id, label, variants: [{label, dockerfile}], notes }>
 */
export async function fetchStacks() {
  return apiFetch("/stacks");
}

/**
 * Fetch the current job queue.
 * Returns: Array<Job>
 */
export async function getQueue() {
  return apiFetch("/queue");
}

/**
 * Enqueue a single run (legacy).
 * @param {{ stackId, variant, suiteName?, durationSec, vus, mix, dockerfileOverride? }} params
 */
export async function enqueueRun(params) {
  return apiFetch("/queue", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(params),
  });
}

/**
 * Enqueue a named suite: cartesian product of stackIds × variants.
 * @param {{ suiteName, stackIds, variants, durationSec, vus, mix, dockerfileOverride? }} params
 * @returns {Promise<{ suiteName, count, jobs }>}
 */
export async function enqueueSuite(params) {
  try {
    return await apiFetch("/queue", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
  } catch (err) {
    // Older control servers only accept single stackId/variant per request.
    if (
      err.message === "stackId and variant are required" &&
      params.stackIds?.length &&
      params.variants?.length
    ) {
      const { stackIds, variants, suiteName, ...shared } = params;
      const jobs = [];
      for (const stackId of stackIds) {
        for (const variant of variants) {
          jobs.push(await enqueueRun({ stackId, variant, suiteName, ...shared }));
        }
      }
      return { suiteName, count: jobs.length, jobs };
    }
    throw err;
  }
}

/**
 * Cancel a pending job.
 * @param {string} id
 */
export async function cancelJob(id) {
  return apiFetch(`/queue/${id}`, { method: "DELETE" });
}

/**
 * Remove all finished jobs from the queue (done, failed, canceled).
 * Does not delete result directories on disk.
 */
export async function clearCompletedJobs() {
  return apiFetch("/queue/completed", { method: "DELETE" });
}

/**
 * Assign a suite name to existing runs.
 * @param {{ runIds: string[], suiteName: string }} params
 */
export async function assignRunsToSuite({ runIds, suiteName }) {
  return apiFetch("/runs/assign-suite", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ runIds, suiteName }),
  });
}

/**
 * Delete one or more completed runs (removes result directories).
 * @param {{ runIds: string[] }} params
 */
export async function deleteRuns({ runIds }) {
  return apiFetch("/runs", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ runIds }),
  });
}

/**
 * Remove suite label from all runs in a suite (runs remain on disk).
 * @param {string} suiteName
 */
export async function dissolveSuite(suiteName) {
  return apiFetch(`/suites/${encodeURIComponent(suiteName)}`, { method: "DELETE" });
}

/**
 * Delete all run directories belonging to a suite.
 * @param {string} suiteName
 */
export async function deleteSuiteRuns(suiteName) {
  return apiFetch(`/suites/${encodeURIComponent(suiteName)}?action=delete-runs`, {
    method: "DELETE",
  });
}

/**
 * Check whether the control server is reachable.
 */
export async function pingServer() {
  try {
    await apiFetch("/queue");
    return true;
  } catch {
    return false;
  }
}

/**
 * Subscribe to the SSE event stream.
 * @param {(event: {type: string, [key: string]: any}) => void} onMessage
 * @param {{ onOpen?: () => void, onClose?: () => void }} [options]
 * @returns {() => void} unsubscribe function
 */
export function subscribeEvents(onMessage, options = {}) {
  const { onOpen, onClose } = options;
  let es;
  let closed = false;
  let reconnectTimer;
  let reconnectDelay = 1000;

  function connect() {
    es = new EventSource(`${BASE}/events`);
    es.onopen = () => {
      reconnectDelay = 1000;
      onOpen?.();
    };
    es.onmessage = (e) => {
      try { onMessage(JSON.parse(e.data)); } catch (_) {}
    };
    es.onerror = () => {
      onClose?.();
      es.close();
      if (!closed) {
        reconnectTimer = setTimeout(connect, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, 15000);
      }
    };
  }

  connect();

  return () => {
    closed = true;
    clearTimeout(reconnectTimer);
    es?.close();
  };
}
