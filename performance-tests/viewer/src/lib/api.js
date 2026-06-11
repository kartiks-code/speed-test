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
 * Enqueue a new run.
 * @param {{ stackId, variant, durationSec, vus, mix: {create,read,update,delete}, dockerfileOverride? }} params
 */
export async function enqueueRun(params) {
  return apiFetch("/queue", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(params),
  });
}

/**
 * Cancel a pending job.
 * @param {string} id
 */
export async function cancelJob(id) {
  return apiFetch(`/queue/${id}`, { method: "DELETE" });
}

/**
 * Subscribe to the SSE event stream.
 * @param {(event: {type: string, [key: string]: any}) => void} onMessage
 * @returns {() => void} unsubscribe function
 */
export function subscribeEvents(onMessage) {
  const es = new EventSource(`${BASE}/events`);
  es.onmessage = (e) => {
    try { onMessage(JSON.parse(e.data)); } catch (_) {}
  };
  return () => es.close();
}
