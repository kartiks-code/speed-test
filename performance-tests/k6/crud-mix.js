/**
 * k6 weighted CRUD mix script for the Petstore API.
 *
 * Parameterized via environment variables:
 *   BASE_URL    — e.g. http://speed-test-go:8080     (required)
 *   BASE_PATH   — e.g. /api/v3  (default: /api/v3)
 *   VUS         — virtual users (default: 20)
 *   DURATION    — k6 duration string (default: 60s)
 *   AUTH_HEADER — optional Authorization header value (e.g. "Bearer token")
 *
 *   MIX_CREATE  — relative weight for Create operations (default: 25)
 *   MIX_READ    — relative weight for Read operations   (default: 25)
 *   MIX_UPDATE  — relative weight for Update operations (default: 25)
 *   MIX_DELETE  — relative weight for Delete operations (default: 25)
 *
 * Weights are relative integers; they are normalised at startup. All-zero
 * defaults to equal 25/25/25/25.
 *
 * setup() seeds a shared pool of 50 pets so that Read/Update/Delete have
 * live targets immediately. Each VU also maintains its own per-iteration
 * pet pool to avoid empty-pool starvation.
 *
 * When SKIP_SETUP=1 (set by run.sh after it pre-seeds pets via curl), setup()
 * returns immediately so k6 only runs the configured load duration and the
 * resource sampler is not inflated by sequential seed requests.
 */

import http from "k6/http";
import { check, sleep } from "k6";
import { SharedArray } from "k6/data";

// ── configuration ─────────────────────────────────────────────────────────────

const BASE_URL    = __ENV.BASE_URL  || "http://localhost:8080";
const BASE_PATH   = __ENV.BASE_PATH !== undefined ? __ENV.BASE_PATH : "/api/v3";
const VUS         = parseInt(__ENV.VUS      || "20", 10);
const DURATION    = __ENV.DURATION          || "60s";
const AUTH_HEADER = __ENV.AUTH_HEADER       || "";

const wCreate = Math.max(0, parseInt(__ENV.MIX_CREATE || "25", 10));
const wRead   = Math.max(0, parseInt(__ENV.MIX_READ   || "25", 10));
const wUpdate = Math.max(0, parseInt(__ENV.MIX_UPDATE || "25", 10));
const wDelete = Math.max(0, parseInt(__ENV.MIX_DELETE || "25", 10));

// Normalise weights into cumulative thresholds for random selection.
// If all weights are 0 fall back to equal distribution.
const total = wCreate + wRead + wUpdate + wDelete || 100;
const threshCreate = wCreate / total;
const threshRead   = threshCreate + wRead / total;
const threshUpdate = threshRead   + wUpdate / total;
// threshDelete = 1.0 (implied — everything above threshUpdate)

export const options = {
  vus: VUS,
  duration: DURATION,
  thresholds: {
    http_req_failed:   [{ threshold: "rate<0.01",  abortOnFail: false }],
    http_req_duration: [{ threshold: "p(95)<500",  abortOnFail: false }],
  },
  summaryTrendStats: ["min", "avg", "med", "p(90)", "p(95)", "p(99)", "max", "count"],
};

// ── headers ───────────────────────────────────────────────────────────────────

const HEADERS = {
  "Content-Type": "application/json",
  "api_key": "benchmark",
  ...(AUTH_HEADER ? { "Authorization": AUTH_HEADER } : {}),
};

// ── helpers ───────────────────────────────────────────────────────────────────

function url(path) {
  return `${BASE_URL}${BASE_PATH}${path}`;
}

function randomName() {
  return `pet-${Math.random().toString(36).slice(2, 9)}`;
}

function randomItem(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

// ── setup: seed a shared pool of pets ────────────────────────────────────────

const SEED_SIZE = 50;

export function setup() {
  if (__ENV.SKIP_SETUP === "1") {
    return { seedIds: [] };
  }

  const ids = [];
  for (let i = 0; i < SEED_SIZE; i++) {
    const payload = JSON.stringify({
      name: randomName(),
      status: "available",
      photoUrls: ["http://example.com/photo.jpg"],
      category: { id: 1, name: "Dogs" },
      tags: [{ id: 1, name: "k6-mix" }],
    });
    const res = http.post(url("/pet"), payload, { headers: HEADERS });
    if (res.status === 200) {
      try {
        const pet = res.json();
        if (pet.id) ids.push(pet.id);
      } catch (_) {}
    }
  }
  return { seedIds: ids };
}

// ── per-VU state (shared-array trick keeps a mutable local pool) ──────────────

// Each VU gets its own copy of the seed list on first call; it extends/shrinks
// the list as Creates/Deletes happen.
const vuPool = {};

function getPool(vuId, seedIds) {
  if (!vuPool[vuId]) {
    vuPool[vuId] = seedIds.slice();
  }
  return vuPool[vuId];
}

// ── operations ────────────────────────────────────────────────────────────────

function doCreate(pool) {
  const payload = JSON.stringify({
    name: randomName(),
    status: "available",
    photoUrls: ["http://example.com/photo.jpg"],
    category: { id: 1, name: "Dogs" },
    tags: [{ id: 1, name: "k6-mix" }],
  });
  const res = http.post(url("/pet"), payload, { headers: HEADERS });
  check(res, { "create pet 200": (r) => r.status === 200 });
  if (res.status === 200) {
    try {
      const pet = res.json();
      if (pet.id) pool.push(pet.id);
    } catch (_) {}
  }
}

function doRead(pool) {
  if (pool.length === 0) return doCreate(pool);

  // Alternate between GET /pet/{id} and GET /pet/findByStatus
  if (Math.random() < 0.7) {
    const petId = randomItem(pool);
    const res = http.get(url(`/pet/${petId}`), { headers: HEADERS });
    check(res, { "get pet 200": (r) => r.status === 200 || r.status === 404 });
  } else {
    const res = http.get(url("/pet/findByStatus?status=available"), { headers: HEADERS });
    check(res, { "findByStatus 200": (r) => r.status === 200 });
  }
}

function doUpdate(pool) {
  if (pool.length === 0) return doCreate(pool);

  const petId = randomItem(pool);
  const payload = JSON.stringify({
    id: petId,
    name: randomName() + "-updated",
    status: "pending",
    photoUrls: ["http://example.com/photo.jpg"],
    category: { id: 1, name: "Dogs" },
    tags: [{ id: 1, name: "k6-mix" }],
  });
  const res = http.put(url("/pet"), payload, { headers: HEADERS });
  check(res, { "update pet 200": (r) => r.status === 200 });
}

function doDelete(pool) {
  if (pool.length === 0) return doCreate(pool);

  // Pop from the end to avoid repeated re-tries on same id
  const idx = Math.floor(Math.random() * pool.length);
  const petId = pool.splice(idx, 1)[0];
  const res = http.del(url(`/pet/${petId}`), null, { headers: HEADERS });
  // 200 or 404 both acceptable (concurrent deletes may race)
  check(res, { "delete pet 200/404": (r) => r.status === 200 || r.status === 404 });
}

// ── main scenario ─────────────────────────────────────────────────────────────

export default function (data) {
  const pool = getPool(__VU, data.seedIds || []);

  const r = Math.random();
  if (r < threshCreate) {
    doCreate(pool);
  } else if (r < threshRead) {
    doRead(pool);
  } else if (r < threshUpdate) {
    doUpdate(pool);
  } else {
    doDelete(pool);
  }

  sleep(0.05);
}
