import { durationsForRuns, variantsForRuns, suiteNames } from "./data.js";

/** Parse run timestamp (20260611T024242Z) to Date. */
export function parseRunTimestamp(ts) {
  if (!ts || typeof ts !== "string") return null;
  const m = ts.match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})Z$/);
  if (!m) return null;
  const [, y, mo, d, h, mi, s] = m;
  return new Date(Date.UTC(+y, +mo - 1, +d, +h, +mi, +s));
}

/** Format run timestamp for display. */
export function formatRunTimestamp(ts) {
  const dt = parseRunTimestamp(ts);
  if (!dt) return ts ?? "—";
  return dt.toISOString().replace("T", " ").replace(".000Z", " UTC");
}

/** Convert datetime-local value to comparable run timestamp string (UTC). */
export function dateInputToRunTimestamp(value) {
  if (!value) return null;
  const dt = new Date(value);
  if (Number.isNaN(dt.getTime())) return null;
  const pad = (n) => String(n).padStart(2, "0");
  return (
    `${dt.getUTCFullYear()}${pad(dt.getUTCMonth() + 1)}${pad(dt.getUTCDate())}` +
    `T${pad(dt.getUTCHours())}${pad(dt.getUTCMinutes())}${pad(dt.getUTCSeconds())}Z`
  );
}

/** Unique stack labels from runs, sorted. */
export function stackLabelsFromRuns(runs) {
  const seen = new Set();
  for (const r of runs) if (r.label) seen.add(r.label);
  return [...seen].sort((a, b) => a.localeCompare(b));
}

/** Unique VU counts from runs, sorted numerically. */
export function vusOptionsFromRuns(runs) {
  const seen = new Set();
  for (const r of runs) if (r.vus != null) seen.add(r.vus);
  return [...seen].sort((a, b) => a - b);
}

export const TIME_PRESETS = [
  { id: "all", label: "Any time" },
  { id: "24h", label: "Last 24 hours" },
  { id: "7d", label: "Last 7 days" },
  { id: "30d", label: "Last 30 days" },
];

export function timestampAfterForPreset(presetId) {
  if (!presetId || presetId === "all") return null;
  const now = Date.now();
  const ms =
    presetId === "24h" ? 24 * 60 * 60 * 1000
    : presetId === "7d" ? 7 * 24 * 60 * 60 * 1000
    : presetId === "30d" ? 30 * 24 * 60 * 60 * 1000
    : null;
  if (!ms) return null;
  const dt = new Date(now - ms);
  const pad = (n) => String(n).padStart(2, "0");
  return (
    `${dt.getUTCFullYear()}${pad(dt.getUTCMonth() + 1)}${pad(dt.getUTCDate())}` +
    `T${pad(dt.getUTCHours())}${pad(dt.getUTCMinutes())}${pad(dt.getUTCSeconds())}Z`
  );
}

/**
 * Filter runs using the same dimensions as Compare (stack, duration, variant, suite)
 * plus optional VU count and time range.
 */
export function filterRuns(runs, filters) {
  const {
    suiteQuery = "",
    stacks = [],
    durations = [],
    variants = [],
    vus = [],
    timestampAfter = null,
    timestampBefore = null,
    unsuitedOnly = false,
  } = filters;

  const q = suiteQuery.trim().toLowerCase();

  return runs.filter((r) => {
    if (unsuitedOnly && r.suite) return false;
    if (q) {
      if (!r.suite || !r.suite.toLowerCase().includes(q)) return false;
    }
    if (stacks.length && !stacks.includes(r.label)) return false;
    if (durations.length && !durations.includes(r.duration)) return false;
    if (variants.length && !variants.includes(r.variant)) return false;
    if (vus.length && !vus.includes(r.vus)) return false;
    if (timestampAfter && r.timestamp < timestampAfter) return false;
    if (timestampBefore && r.timestamp > timestampBefore) return false;
    return true;
  });
}

export function buildFilterOptions(runs) {
  return {
    suiteNames: suiteNames(runs),
    stacks: stackLabelsFromRuns(runs),
    durations: durationsForRuns(runs),
    variants: variantsForRuns(runs),
    vus: vusOptionsFromRuns(runs),
  };
}
