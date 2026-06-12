export const SUITE_PAGE_SIZE = 6;

const STORAGE_KEY = "perf-viewer-suite-groups";

/** Stable key for a stack × variant combo within a suite. */
export function stackVariantKey(run) {
  return `${run.stack_id}:${run.variant}`;
}

export function stackVariantLabel(run) {
  return `${run.label} · ${run.variant}`;
}

/** One run per stack×variant (most recent timestamp wins). */
export function uniqueStackVariantRuns(suiteRuns) {
  const byKey = new Map();
  for (const run of suiteRuns) {
    const key = stackVariantKey(run);
    const existing = byKey.get(key);
    if (!existing || run.timestamp > existing.timestamp) {
      byKey.set(key, run);
    }
  }
  return [...byKey.values()].sort((a, b) => {
    const labelCmp = a.label.localeCompare(b.label);
    if (labelCmp !== 0) return labelCmp;
    return a.variant.localeCompare(b.variant);
  });
}

export function defaultPageAssignments(runs, pageSize = SUITE_PAGE_SIZE) {
  const assignments = {};
  runs.forEach((run, i) => {
    assignments[stackVariantKey(run)] = Math.floor(i / pageSize) + 1;
  });
  return assignments;
}

function readAllStored() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

export function loadSuitePageAssignments(suiteName, runs) {
  const defaults = defaultPageAssignments(runs);
  const stored = readAllStored()[suiteName];
  if (!stored) return defaults;

  const merged = { ...defaults };
  for (const run of runs) {
    const key = stackVariantKey(run);
    if (stored[key] != null) merged[key] = stored[key];
  }
  return merged;
}

export function saveSuitePageAssignments(suiteName, assignments) {
  const all = readAllStored();
  all[suiteName] = assignments;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
}

/** Split runs into ordered pages using page-number assignments. */
export function assignmentsToPages(runs, assignments) {
  const pageMap = {};
  for (const run of runs) {
    const page = assignments[stackVariantKey(run)] ?? 1;
    if (!pageMap[page]) pageMap[page] = [];
    pageMap[page].push(run);
  }

  return Object.keys(pageMap)
    .map(Number)
    .sort((a, b) => a - b)
    .map((pageNumber) => ({
      pageNumber,
      runs: pageMap[pageNumber].sort((a, b) => {
        const labelCmp = a.label.localeCompare(b.label);
        if (labelCmp !== 0) return labelCmp;
        return a.variant.localeCompare(b.variant);
      }),
    }));
}

export function pageCountFromAssignments(assignments) {
  const values = Object.values(assignments);
  return values.length ? Math.max(...values) : 1;
}

export function selectionsFromRuns(runs) {
  return runs.map((r) => ({
    stackLabel: r.label,
    duration: r.duration,
    variant: r.variant,
    runId: r.run_id,
  }));
}
