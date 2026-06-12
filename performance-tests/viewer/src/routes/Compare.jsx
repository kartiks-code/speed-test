import React, { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  loadIndex, loadRun, groupByStack, groupBySuite, suiteNames,
  durationsForRuns, variantsForRuns, compareRunLabel, mixPercentages, seriesColor,
} from "../lib/data.js";
import {
  uniqueStackVariantRuns,
  loadSuitePageAssignments,
  saveSuitePageAssignments,
  assignmentsToPages,
  selectionsFromRuns,
  SUITE_PAGE_SIZE,
} from "../lib/suiteGroups.js";
import { useDataRefresh } from "../lib/DataRefreshContext.jsx";
import SuiteGroupModal from "../components/SuiteGroupModal.jsx";
import {
  RpsCompareChart,
  LatencyCompareChart,
  ErrorRateCompareChart,
  CpuCompareChart,
  RamCompareChart,
  PgCompareChart,
} from "../components/CompareCharts.jsx";
import { CpuOverlayChart, RamOverlayChart } from "../components/TimeSeriesChart.jsx";

const MAX_MANUAL_SERIES = 6;

const MIX_BAR_OPS = [
  { key: "create", color: "#22c55e" },
  { key: "read", color: "#f59e0b" },
  { key: "update", color: "#6366f1" },
  { key: "delete", color: "#ef4444" },
];

function MixBarMini({ mix }) {
  const pct = mixPercentages(mix);
  if (!pct) return null;
  return (
    <div className="mix-bar mini">
      {MIX_BAR_OPS.map(({ key, color }) => {
        const w = pct[key];
        return w > 0 ? (
          <div key={key} title={`${key} ${w}%`} style={{ width: `${w}%`, background: color }} />
        ) : null;
      })}
    </div>
  );
}

function RunBriefSummary({ run, compact = false }) {
  if (!run) return null;
  const pct = mixPercentages(run.mix);
  const durationSec = run.duration?.replace(/s$/, "") ?? run.duration ?? "—";

  return (
    <div className={`run-selector-summary${compact ? " compact" : ""}`}>
      <span>{run.vus ?? "—"} VU</span>
      <span className="run-selector-summary-sep">·</span>
      <span>{durationSec}s</span>
      {pct ? (
        <>
          <span className="run-selector-summary-sep">·</span>
          <span className="run-selector-mix-text">
            C{pct.create} R{pct.read} U{pct.update} D{pct.delete}
          </span>
          <MixBarMini mix={run.mix} />
        </>
      ) : (
        <>
          <span className="run-selector-summary-sep">·</span>
          <span className="run-selector-mix-text">Full CRUD cycle</span>
        </>
      )}
    </div>
  );
}

function pickRunForFilters(runs, duration, variant) {
  return runs.find((r) => r.duration === duration && r.variant === variant)?.run_id ?? "";
}

// A single selector row: color dot · #N · Stack · Run time · Variant · Run · ×
function RunSelectorRow({ index, stacks, stackKeys, selection, onChange, onRemove }) {
  const { stackLabel, duration, variant, runId } = selection;
  const allRuns = stacks[stackLabel] || [];
  const durations = durationsForRuns(allRuns);
  const runsForDuration = allRuns.filter((r) => r.duration === duration);
  const variants = variantsForRuns(runsForDuration);
  const runsForVariant = runsForDuration.filter((r) => r.variant === variant);
  const selectedIndexRun = runId ? allRuns.find((r) => r.run_id === runId) : null;

  function handleStackChange(newStack) {
    const runs = stacks[newStack] || [];
    const durs = durationsForRuns(runs);
    const newDur = durs[0] ?? "";
    const runsAtDur = runs.filter((r) => r.duration === newDur);
    const vars = variantsForRuns(runsAtDur);
    const newVar = vars[0] ?? "";
    const newRunId = pickRunForFilters(runs, newDur, newVar);
    onChange({ stackLabel: newStack, duration: newDur, variant: newVar, runId: newRunId });
  }

  function handleDurationChange(newDur) {
    const runsAtDur = allRuns.filter((r) => r.duration === newDur);
    const vars = variantsForRuns(runsAtDur);
    const newVar = vars.includes(variant) ? variant : (vars[0] ?? "");
    const newRunId = pickRunForFilters(allRuns, newDur, newVar);
    onChange({ stackLabel, duration: newDur, variant: newVar, runId: newRunId });
  }

  function handleVariantChange(newVar) {
    const newRunId = pickRunForFilters(allRuns, duration, newVar);
    onChange({ stackLabel, duration, variant: newVar, runId: newRunId });
  }

  function handleRunChange(newRunId) {
    onChange({ stackLabel, duration, variant, runId: newRunId });
  }

  return (
    <div className="run-selector-row">
      <div className="run-selector-main">
        <div className="run-selector-dot" style={{ background: seriesColor(index) }} />
        <span className="run-selector-label">#{index + 1}</span>

        <div className="control-group" style={{ margin: 0 }}>
          <select
            className="control-select"
            style={{ minWidth: 190 }}
            value={stackLabel}
            onChange={(e) => handleStackChange(e.target.value)}
          >
            <option value="">— stack —</option>
            {stackKeys.map((k) => <option key={k} value={k}>{k}</option>)}
          </select>
        </div>

        <div className="control-group" style={{ margin: 0 }}>
          <select
            className="control-select"
            style={{ minWidth: 90 }}
            value={duration}
            onChange={(e) => handleDurationChange(e.target.value)}
            disabled={!durations.length}
          >
            {!durations.length && <option value="">—</option>}
            {durations.map((d) => <option key={d} value={d}>{d}</option>)}
          </select>
        </div>

        <div className="control-group" style={{ margin: 0 }}>
          <select
            className="control-select"
            style={{ minWidth: 110 }}
            value={variant}
            onChange={(e) => handleVariantChange(e.target.value)}
            disabled={!variants.length}
          >
            {!variants.length && <option value="">—</option>}
            {variants.map((v) => <option key={v} value={v}>{v}</option>)}
          </select>
        </div>

        <div className="control-group" style={{ margin: 0 }}>
          <select
            className="control-select"
            style={{ minWidth: 160 }}
            value={runId}
            onChange={(e) => handleRunChange(e.target.value)}
            disabled={!runsForVariant.length}
          >
            {!runsForVariant.length && <option value="">—</option>}
            {runsForVariant.map((r) => (
              <option key={r.run_id} value={r.run_id}>{compareRunLabel(r)}</option>
            ))}
          </select>
        </div>

        <button className="run-selector-remove" onClick={onRemove} title="Remove">×</button>
      </div>

      <RunBriefSummary run={selectedIndexRun} />
    </div>
  );
}

function applySuiteGrouping(name, suiteRuns, setters) {
  const unique = uniqueStackVariantRuns(suiteRuns);
  const assignments = loadSuitePageAssignments(name, unique);
  const pages = assignmentsToPages(unique, assignments);
  const firstPage = pages[0]?.pageNumber ?? 1;
  setters.setSuiteFilter(name);
  setters.setSuiteMode(true);
  setters.setSuitePageAssignments(assignments);
  setters.setCurrentSuitePage(firstPage);
  setters.setSelections(selectionsFromRuns(pages[0]?.runs ?? []));
}

export default function Compare() {
  const [searchParams] = useSearchParams();
  const { refreshToken, lastRunId } = useDataRefresh();
  const [stacks, setStacks] = useState({});
  const [stackKeys, setStackKeys] = useState([]);
  const [suites, setSuites] = useState({});
  const [suiteNameList, setSuiteNameList] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const [suiteFilter, setSuiteFilter] = useState("");
  const [suiteMode, setSuiteMode] = useState(false);
  const [suitePageAssignments, setSuitePageAssignments] = useState({});
  const [currentSuitePage, setCurrentSuitePage] = useState(1);
  const [showGroupModal, setShowGroupModal] = useState(false);

  const [selections, setSelections] = useState([
    { stackLabel: "", duration: "", variant: "", runId: "" },
    { stackLabel: "", duration: "", variant: "", runId: "" },
  ]);

  const [runDetails, setRunDetails] = useState({});

  useEffect(() => {
    const deepLinkSuite = searchParams.get("suite");

    loadIndex({ force: refreshToken > 0 })
      .then((idx) => {
        const grouped = groupByStack(idx.runs);
        const keys = Object.keys(grouped).sort();
        const bySuite = groupBySuite(idx.runs);
        const names = suiteNames(idx.runs);

        setStacks(grouped);
        setStackKeys(keys);
        setSuites(bySuite);
        setSuiteNameList(names);

        if (deepLinkSuite && bySuite[deepLinkSuite]) {
          applySuiteGrouping(deepLinkSuite, bySuite[deepLinkSuite], {
            setSuiteFilter,
            setSuiteMode,
            setSuitePageAssignments,
            setCurrentSuitePage,
            setSelections,
          });
        } else if (refreshToken === 0 && !deepLinkSuite) {
          setSelections((prev) =>
            prev.map((sel, i) => {
              if (sel.stackLabel || !keys[i]) return sel;
              const sl = keys[i];
              const runs = grouped[sl] || [];
              const durs = durationsForRuns(runs);
              const dur = durs[0] ?? "";
              const runsAtDur = runs.filter((r) => r.duration === dur);
              const vars = variantsForRuns(runsAtDur);
              const variant = vars[0] ?? "";
              const runId = pickRunForFilters(runs, dur, variant);
              return { stackLabel: sl, duration: dur, variant, runId };
            })
          );
        }
        setLoading(false);
      })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, [refreshToken, searchParams]);

  function handleSuiteFilterChange(value) {
    setSuiteFilter(value);
    const exact = suiteNameList.find((n) => n.toLowerCase() === value.trim().toLowerCase());
    if (exact && suites[exact]) {
      applySuiteGrouping(exact, suites[exact], {
        setSuiteFilter,
        setSuiteMode,
        setSuitePageAssignments,
        setCurrentSuitePage,
        setSelections,
      });
    } else if (!value.trim()) {
      setSuiteMode(false);
      setSuitePageAssignments({});
      setCurrentSuitePage(1);
    }
  }

  function loadSuiteByName(name) {
    const runs = suites[name];
    if (!runs?.length) return;
    applySuiteGrouping(name, runs, {
      setSuiteFilter,
      setSuiteMode,
      setSuitePageAssignments,
      setCurrentSuitePage,
      setSelections,
    });
  }

  function clearSuite() {
    setSuiteFilter("");
    setSuiteMode(false);
    setSuitePageAssignments({});
    setCurrentSuitePage(1);
    setSelections([
      { stackLabel: stackKeys[0] ?? "", duration: "", variant: "", runId: "" },
      { stackLabel: stackKeys[1] ?? "", duration: "", variant: "", runId: "" },
    ]);
  }

  function goToSuitePage(pageNumber) {
    const runs = suites[suiteFilter];
    if (!runs?.length) return;
    const unique = uniqueStackVariantRuns(runs);
    const pages = assignmentsToPages(unique, suitePageAssignments);
    const page = pages.find((p) => p.pageNumber === pageNumber);
    if (!page) return;
    setCurrentSuitePage(pageNumber);
    setSelections(selectionsFromRuns(page.runs));
  }

  function handleSaveGroups(newAssignments) {
    saveSuitePageAssignments(suiteFilter, newAssignments);
    setSuitePageAssignments(newAssignments);
    setShowGroupModal(false);
    const runs = suites[suiteFilter];
    if (!runs?.length) return;
    const unique = uniqueStackVariantRuns(runs);
    const pages = assignmentsToPages(unique, newAssignments);
    const stillOnPage = pages.some((p) => p.pageNumber === currentSuitePage);
    const targetPage = stillOnPage ? currentSuitePage : (pages[0]?.pageNumber ?? 1);
    const page = pages.find((p) => p.pageNumber === targetPage);
    if (!page) return;
    setCurrentSuitePage(targetPage);
    setSelections(selectionsFromRuns(page.runs));
  }

  const filteredSuiteNames = suiteNameList.filter((n) =>
    !suiteFilter.trim() || n.toLowerCase().includes(suiteFilter.trim().toLowerCase())
  );

  const suiteUniqueRuns = suiteMode && suites[suiteFilter]
    ? uniqueStackVariantRuns(suites[suiteFilter])
    : [];
  const suitePages = suiteMode && suiteUniqueRuns.length
    ? assignmentsToPages(suiteUniqueRuns, suitePageAssignments)
    : [];
  const suitePageIndex = suitePages.findIndex((p) => p.pageNumber === currentSuitePage);
  const suiteNeedsPaging = suiteUniqueRuns.length > SUITE_PAGE_SIZE;

  // Load detail JSON for any newly selected run
  useEffect(() => {
    for (const { runId } of selections) {
      if (!runId) continue;
      const force = refreshToken > 0 && runId === lastRunId;
      if (!force && runDetails[runId]) continue;
      loadRun(runId, { force })
        .then((r) => setRunDetails((prev) => ({ ...prev, [runId]: r })))
        .catch(() => {});
    }
  }, [selections, refreshToken, lastRunId]);

  const updateSelection = (i, val) => {
    setSuiteMode(false);
    setSelections((prev) => prev.map((s, idx) => (idx === i ? val : s)));
  };
  const removeSelection = (i) => {
    setSuiteMode(false);
    setSelections((prev) => prev.filter((_, idx) => idx !== i));
  };
  const addSelection = () => {
    setSuiteMode(false);
    setSelections((prev) => [...prev, { stackLabel: "", duration: "", variant: "", runId: "" }]);
  };

  if (loading) return <div className="loading">Loading index…</div>;
  if (error) return <div className="loading" style={{ color: "#ef4444" }}>Error: {error}</div>;

  const labelledSeries = selections
    .map((sel, i) => ({
      index: i,
      color: seriesColor(i),
      run: sel.runId ? runDetails[sel.runId] : null,
    }))
    .filter((s) => s.run != null)
    .map((s) => ({
      ...s,
      label: `${s.run.meta.label} (${s.run.meta.variant}, ${s.run.meta.duration})`,
    }));

  const hasSeries = labelledSeries.length >= 1;

  const cpuOverlaySeries = labelledSeries.map((s) => ({
    label: s.label,
    color: s.color,
    data: s.run.timeseries || [],
  }));
  const ramOverlaySeries = labelledSeries.map((s) => ({
    label: s.label,
    color: s.color,
    data: s.run.timeseries || [],
  }));

  return (
    <div>
      {/* Suite search / load */}
      <div className="suite-filter-bar">
        <div className="control-group" style={{ flex: 1, minWidth: 220 }}>
          <label className="control-label">Search suite</label>
          <input
            type="search"
            className="control-input"
            placeholder="Type to filter suites…"
            value={suiteFilter}
            onChange={(e) => handleSuiteFilterChange(e.target.value)}
            list="suite-suggestions"
          />
          <datalist id="suite-suggestions">
            {suiteNameList.map((n) => <option key={n} value={n} />)}
          </datalist>
        </div>
        {suiteMode && (
          <div className="suite-mode-badge">
            Suite view · {suiteUniqueRuns.length} run{suiteUniqueRuns.length !== 1 ? "s" : ""}
            {suiteNeedsPaging && suitePages.length > 0 && (
              <span className="suite-page-indicator">
                · Page {currentSuitePage} of {suitePages.length}
              </span>
            )}
            <button type="button" className="btn-link" onClick={clearSuite}>clear</button>
          </div>
        )}
      </div>

      {suiteMode && suiteNeedsPaging && (
        <div className="suite-page-nav">
          <button
            type="button"
            className="btn-secondary btn-sm"
            disabled={suitePageIndex <= 0}
            onClick={() => goToSuitePage(suitePages[suitePageIndex - 1].pageNumber)}
          >
            ← Prev
          </button>
          <div className="suite-page-tabs">
            {suitePages.map((p) => (
              <button
                key={p.pageNumber}
                type="button"
                className={`suite-page-tab${p.pageNumber === currentSuitePage ? " active" : ""}`}
                onClick={() => goToSuitePage(p.pageNumber)}
              >
                {p.pageNumber}
                <span className="suite-page-tab-count">{p.runs.length}</span>
              </button>
            ))}
          </div>
          <button
            type="button"
            className="btn-secondary btn-sm"
            disabled={suitePageIndex < 0 || suitePageIndex >= suitePages.length - 1}
            onClick={() => goToSuitePage(suitePages[suitePageIndex + 1].pageNumber)}
          >
            Next →
          </button>
          <button
            type="button"
            className="btn-secondary btn-sm"
            onClick={() => setShowGroupModal(true)}
          >
            Change groups
          </button>
        </div>
      )}

      {filteredSuiteNames.length > 0 && suiteFilter.trim() && !suiteMode && (
        <div className="suite-suggestions">
          {filteredSuiteNames.slice(0, 8).map((name) => (
            <button
              key={name}
              type="button"
              className="suite-suggestion-btn"
              onClick={() => loadSuiteByName(name)}
            >
              {name}
              <span className="suite-suggestion-count">{suites[name]?.length ?? 0} runs</span>
            </button>
          ))}
        </div>
      )}

      {!suiteMode && (
        <div className="compare-selectors">
          {selections.map((sel, i) => (
            <RunSelectorRow
              key={i}
              index={i}
              stacks={stacks}
              stackKeys={stackKeys}
              selection={sel}
              onChange={(val) => updateSelection(i, val)}
              onRemove={() => removeSelection(i)}
            />
          ))}
          <div>
            <button
              className="add-run-btn"
              onClick={addSelection}
              disabled={selections.length >= MAX_MANUAL_SERIES}
            >
              + Add run
            </button>
          </div>
        </div>
      )}

      {!hasSeries && (
        <div className="empty-state">
          <div className="empty-state-icon">🔀</div>
          <div>Select runs above or search for a suite to compare.</div>
        </div>
      )}

      {showGroupModal && suiteMode && (
        <SuiteGroupModal
          suiteName={suiteFilter}
          runs={suiteUniqueRuns}
          assignments={suitePageAssignments}
          onSave={handleSaveGroups}
          onClose={() => setShowGroupModal(false)}
        />
      )}

      {hasSeries && (
        <>
          <div style={{ display: "flex", gap: 16, marginBottom: 20, flexWrap: "wrap" }}>
            {labelledSeries.map((s) => (
              <div key={s.label} className="compare-series-legend-item">
                <div className="compare-series-legend-label">
                  <span style={{ width: 12, height: 12, borderRadius: "50%", background: s.color, display: "inline-block", flexShrink: 0 }} />
                  <span style={{ color: "#e2e8f0" }}>{s.label}</span>
                </div>
                <RunBriefSummary run={s.run.meta} compact />
              </div>
            ))}
          </div>

          <div className="compare-charts">
            <RpsCompareChart series={labelledSeries} />
            <ErrorRateCompareChart series={labelledSeries} />
            <LatencyCompareChart series={labelledSeries} />
            <CpuCompareChart series={labelledSeries} />
            <RamCompareChart series={labelledSeries} />
            <PgCompareChart series={labelledSeries} />
          </div>

          {cpuOverlaySeries.some((s) => s.data.length > 0) && (
            <div className="compare-charts" style={{ marginTop: 20 }}>
              <CpuOverlayChart series={cpuOverlaySeries} />
              <RamOverlayChart series={ramOverlaySeries} />
            </div>
          )}
        </>
      )}
    </div>
  );
}
