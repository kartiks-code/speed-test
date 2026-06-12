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
import CompareMetricsTable from "../components/CompareMetricsTable.jsx";
import InfoModal, { InfoButton } from "../components/InfoModal.jsx";
import { SECTION_INFO } from "../lib/sectionInfo.js";

const MAX_MANUAL_SERIES = 6;
const COMPARE_VIEW_KEY = "compareViewMode";

function loadCompareViewMode() {
  try {
    const v = localStorage.getItem(COMPARE_VIEW_KEY);
    return v === "table" ? "table" : "chart";
  } catch {
    return "chart";
  }
}

function saveCompareViewMode(mode) {
  try {
    localStorage.setItem(COMPARE_VIEW_KEY, mode);
  } catch {
    /* ignore */
  }
}

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
  const [viewMode, setViewMode] = useState(loadCompareViewMode);

  const [infoSection, setInfoSection] = useState(null);

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

  const runIdsToLoad = viewMode === "table" && suiteMode && suiteUniqueRuns.length
    ? suiteUniqueRuns.map((r) => r.run_id)
    : selections.map((s) => s.runId).filter(Boolean);

  // Load detail JSON for selected runs (current chart page) or full suite in table view
  useEffect(() => {
    for (const runId of runIdsToLoad) {
      const force = refreshToken > 0 && runId === lastRunId;
      if (!force && runDetails[runId]) continue;
      loadRun(runId, { force })
        .then((r) => setRunDetails((prev) => ({ ...prev, [runId]: r })))
        .catch(() => {});
    }
  }, [runIdsToLoad, refreshToken, lastRunId]);

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

  function buildLabelledSeries(items, runIdForItem) {
    return items
      .map((item, i) => {
        const runId = runIdForItem(item);
        const run = runId ? runDetails[runId] : null;
        if (!run) return null;
        return {
          index: i,
          color: seriesColor(i),
          run,
          label: `${run.meta.label} (${run.meta.variant}, ${run.meta.duration}, ${run.meta.vus ?? "?"} VU)`,
        };
      })
      .filter(Boolean);
  }

  const chartSeries = buildLabelledSeries(
    selections.filter((s) => s.runId),
    (sel) => sel.runId,
  );

  const tableSeries = suiteMode && viewMode === "table"
    ? buildLabelledSeries(suiteUniqueRuns, (r) => r.run_id)
    : chartSeries;

  const activeSeries = viewMode === "table" ? tableSeries : chartSeries;
  const expectedSeriesCount = viewMode === "table" && suiteMode
    ? suiteUniqueRuns.length
    : selections.filter((s) => s.runId).length;
  const stillLoadingSeries = expectedSeriesCount > 0 && activeSeries.length < expectedSeriesCount;
  const hasSeries = activeSeries.length >= 1;

  const cpuOverlaySeries = chartSeries.map((s) => ({
    label: s.label,
    color: s.color,
    data: s.run.timeseries || [],
  }));
  const ramOverlaySeries = chartSeries.map((s) => ({
    label: s.label,
    color: s.color,
    data: s.run.timeseries || [],
  }));

  const showSuitePaging = suiteMode && suiteNeedsPaging && viewMode === "chart";
  const showViewToolbar = suiteMode
    ? suiteUniqueRuns.length > 0
    : selections.some((s) => s.runId);

  function toggleViewMode() {
    setViewMode((prev) => {
      const next = prev === "chart" ? "table" : "chart";
      saveCompareViewMode(next);
      return next;
    });
  }

  const viewToggleBtn = (
    <button
      type="button"
      className="btn-secondary btn-sm"
      onClick={toggleViewMode}
    >
      {viewMode === "chart" ? "Switch to table view" : "Switch to chart view"}
    </button>
  );

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
            {showSuitePaging && suitePages.length > 0 && (
              <span className="suite-page-indicator">
                · Page {currentSuitePage} of {suitePages.length}
              </span>
            )}
            {viewMode === "table" && suiteUniqueRuns.length > 0 && (
              <span className="suite-page-indicator">
                · Table: all {suiteUniqueRuns.length} runs
              </span>
            )}
            <button type="button" className="btn-link" onClick={clearSuite}>clear</button>
          </div>
        )}
      </div>

      {showSuitePaging && (
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
          {viewToggleBtn}
        </div>
      )}

      {showViewToolbar && !showSuitePaging && (
        <div className="compare-view-toolbar">
          {viewMode === "table" && suiteMode && (
            <span className="compare-view-toolbar-hint">
              Showing all {suiteUniqueRuns.length} runs
            </span>
          )}
          {viewToggleBtn}
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

      {!hasSeries && !stillLoadingSeries && (
        <div className="empty-state">
          <div className="empty-state-icon">🔀</div>
          <div>Select runs above or search for a suite to compare.</div>
        </div>
      )}

      {stillLoadingSeries && !hasSeries && (
        <div className="loading">Loading run data…</div>
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

      {infoSection && (
        <InfoModal
          sectionInfo={SECTION_INFO[infoSection]}
          onClose={() => setInfoSection(null)}
        />
      )}

      {(hasSeries || stillLoadingSeries) && (
        <>
          {viewMode === "chart" && hasSeries && (
            <>
              <div style={{ display: "flex", gap: 16, marginBottom: 20, flexWrap: "wrap" }}>
                {chartSeries.map((s) => (
                  <div key={s.label} className="compare-series-legend-item">
                    <div className="compare-series-legend-label">
                      <span style={{ width: 12, height: 12, borderRadius: "50%", background: s.color, display: "inline-block", flexShrink: 0 }} />
                      <span style={{ color: "#e2e8f0" }}>{s.label}</span>
                    </div>
                    <RunBriefSummary run={s.run.meta} compact />
                  </div>
                ))}
              </div>

                <div className="compare-section">
                <div className="compare-section-title">
                  <span>Performance Statistics</span>
                  <InfoButton onClick={() => setInfoSection("Performance Statistics")} label="Performance Statistics" />
                </div>
                <div className="compare-charts">
                  <RpsCompareChart series={chartSeries} />
                  <ErrorRateCompareChart series={chartSeries} />
                  <LatencyCompareChart series={chartSeries} />
                </div>
              </div>

              <div className="compare-section">
                <div className="compare-section-title">
                  <span>Resource Usage Metrics</span>
                  <InfoButton onClick={() => setInfoSection("Resource Usage Metrics")} label="Resource Usage Metrics" />
                </div>
                <div className="compare-charts">
                  <RamCompareChart series={chartSeries} />
                  <CpuCompareChart series={chartSeries} />
                </div>
                {cpuOverlaySeries.some((s) => s.data.length > 0) && (
                  <div className="compare-charts" style={{ marginTop: 12 }}>
                    <RamOverlayChart series={ramOverlaySeries} />
                    <CpuOverlayChart series={cpuOverlaySeries} />
                  </div>
                )}
              </div>

              <div className="compare-section">
                <div className="compare-section-title">
                  <span>DB Metrics</span>
                  <InfoButton onClick={() => setInfoSection("DB Metrics")} label="DB Metrics" />
                </div>
                <div className="compare-charts">
                  <PgCompareChart series={chartSeries} />
                </div>
              </div>
            </>
          )}

          {viewMode === "table" && (
            stillLoadingSeries ? (
              <div className="loading">
                Loading run data… ({activeSeries.length}/{expectedSeriesCount})
              </div>
            ) : (
              <CompareMetricsTable series={tableSeries} />
            )
          )}
        </>
      )}
    </div>
  );
}
