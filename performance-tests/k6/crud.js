/**
 * k6 CRUD load script for the Petstore API.
 *
 * Parameterized via environment variables:
 *   BASE_URL   — e.g. http://speed-test-go:8080     (required)
 *   BASE_PATH  — e.g. /api/v3  (default: /api/v3; use empty string for Python)
 *   VUS        — virtual users (default: 20)
 *   DURATION   — k6 duration string (default: 60s)
 *
 * Run (via Docker, as orchestrated by run.sh):
 *   docker run --rm --network database_default \
 *     -e BASE_URL=http://speed-test-go:8080 \
 *     -e BASE_PATH=/api/v3 \
 *     -v $PWD/results:/results \
 *     grafana/k6 run --summary-export /results/k6-summary.json /scripts/crud.js
 *
 * Scenario: each VU iterates a full CRUD cycle:
 *   1. Create a pet           POST   /pet
 *   2. Read the pet           GET    /pet/{id}
 *   3. Update the pet         PUT    /pet
 *   4. Find by status         GET    /pet/findByStatus?status=available
 *   5. Upload a (tiny) file   POST   /pet/{id}/uploadFile
 *   6. Place an order         POST   /store/order
 *   7. Get the order          GET    /store/order/{orderId}
 *   8. Get inventory          GET    /store/inventory
 *   9. Delete the order       DELETE /store/order/{orderId}
 *  10. Delete the pet         DELETE /pet/{id}
 *
 * Thresholds:
 *   p(95) < 500ms, error rate < 1%
 */

import http from "k6/http";
import { check, sleep } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";

// ── configuration ──────────────────────────────────────────────────────────

const BASE_URL     = __ENV.BASE_URL  || "http://localhost:8080";
const BASE_PATH    = __ENV.BASE_PATH !== undefined ? __ENV.BASE_PATH : "/api/v3";
const VUS          = parseInt(__ENV.VUS      || "20", 10);
const DURATION     = __ENV.DURATION          || "60s";
// Optional: add an Authorization header to every request (needed for stacks
// with stub OAuth2 auth that reject unauthenticated requests at the framework level).
const AUTH_HEADER  = __ENV.AUTH_HEADER || "";

export const options = {
  vus: VUS,
  duration: DURATION,
  // Thresholds are informational — abortOnFail is false so a struggling stack doesn't
  // stop the benchmark suite. Violations are visible in k6-summary.json and k6.log.
  thresholds: {
    http_req_failed:   [{ threshold: "rate<0.01",  abortOnFail: false }],
    http_req_duration: [{ threshold: "p(95)<500",  abortOnFail: false }],
  },
  summaryTrendStats: ["min", "avg", "med", "p(90)", "p(95)", "p(99)", "max", "count"],
};

// ── helpers ────────────────────────────────────────────────────────────────

// Always include api_key so spec-strict validators (Node.js express-openapi-validator)
// pass the /store/inventory security requirement without configuration changes.
// AUTH_HEADER is also forwarded for stacks with OAuth2 stubs that enforce Bearer presence.
const HEADERS = {
  "Content-Type": "application/json",
  "api_key": "benchmark",
  ...(AUTH_HEADER ? { "Authorization": AUTH_HEADER } : {}),
};

function url(path) {
  return `${BASE_URL}${BASE_PATH}${path}`;
}

function randomName() {
  return `pet-${Math.random().toString(36).slice(2, 9)}`;
}

// ── main scenario ─────────────────────────────────────────────────────────

export default function () {
  const petName = randomName();

  // 1. Create pet
  const createPayload = JSON.stringify({
    name: petName,
    status: "available",
    photoUrls: ["http://example.com/photo.jpg"],
    category: { id: 1, name: "Dogs" },
    tags: [{ id: 1, name: "k6" }],
  });

  let createRes = http.post(url("/pet"), createPayload, { headers: HEADERS });
  const created = check(createRes, {
    "create pet 200": (r) => r.status === 200,
  });
  if (!created) {
    return;
  }

  let pet;
  try {
    pet = createRes.json();
  } catch (_) {
    return;
  }
  const petId = pet.id;
  if (!petId) return;

  // 2. Read pet
  const getRes = http.get(url(`/pet/${petId}`), { headers: HEADERS });
  check(getRes, { "get pet 200": (r) => r.status === 200 });

  // 3. Update pet
  const updatePayload = JSON.stringify({
    id: petId,
    name: petName + "-updated",
    status: "pending",
    photoUrls: ["http://example.com/photo.jpg"],
    category: { id: 1, name: "Dogs" },
    tags: [{ id: 1, name: "k6" }],
  });
  const updateRes = http.put(url("/pet"), updatePayload, { headers: HEADERS });
  check(updateRes, { "update pet 200": (r) => r.status === 200 });

  // 4. Find by status
  const findRes = http.get(url("/pet/findByStatus?status=available"), { headers: HEADERS });
  check(findRes, { "findByStatus 200": (r) => r.status === 200 });

  // 5. Upload a minimal file (raw bytes)
  const fileRes = http.post(
    url(`/pet/${petId}/uploadImage`),
    "fake-image-bytes",
    { headers: { "Content-Type": "application/octet-stream" } }
  );
  // 200 or 415 are both acceptable depending on implementation
  check(fileRes, { "uploadFile 2xx/4xx": (r) => r.status < 500 });

  // 6. Place order
  const orderPayload = JSON.stringify({
    petId: petId,
    quantity: 1,
    status: "placed",
    complete: false,
  });
  const orderRes = http.post(url("/store/order"), orderPayload, { headers: HEADERS });
  check(orderRes, { "place order 200": (r) => r.status === 200 });

  let orderId = null;
  try {
    orderId = orderRes.json().id;
  } catch (_) {}

  if (orderId) {
    // 7. Get order
    const getOrderRes = http.get(url(`/store/order/${orderId}`), { headers: HEADERS });
    check(getOrderRes, { "get order 200": (r) => r.status === 200 });

    // 8. Inventory
    const invRes = http.get(url("/store/inventory"), { headers: HEADERS });
    check(invRes, { "inventory 200": (r) => r.status === 200 });

    // 9. Delete order
    const delOrderRes = http.del(url(`/store/order/${orderId}`), null, { headers: HEADERS });
    check(delOrderRes, { "delete order 200": (r) => r.status === 200 });
  }

  // 10. Delete pet
  const delPetRes = http.del(url(`/pet/${petId}`), null, { headers: HEADERS });
  check(delPetRes, { "delete pet 200": (r) => r.status === 200 });

  sleep(0.05);
}
