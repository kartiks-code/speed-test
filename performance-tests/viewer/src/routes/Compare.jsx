import React, { useEffect, useState } from "react";
import { loadIndex, loadRun, groupByStack, durationsForRuns, runLabel, seriesColor } from "../lib/data.js";
import {
  RpsCompareChart,
  LatencyCompareChart,
  ErrorRateCompareChart,
  CpuCompareChart,
  RamCompareChart,
  PgCompareChart,
} from "../components/CompareCharts.jsx";
import { CpuOverlayChart, RamOverlayChart } from "../components/TimeSeriesChart.jsx";

const MAX_SERIES = 4;

// A single selector row: color dot · #N · Stack · Run time · Run · ×
function RunSelectorRow({ index, stacks, stackKeys, selection, onChange, onRemove }) {
  const { stackLabel, duration, runId } = selection;
  const allRuns = stacks[stackLabel] || [];
  const durations = durationsForRuns(allRuns);
  const runsForDuration = allRuns.filter((r) => r.duration === duration);

  function handleStackChange(newStack) {
    const runs = stacks[newStack] || [];
    const durs = durationsForRuns(runs);
    const newDur = durs[0] ?? "";
    const newRunId = runs.filter((r) => r.duration === newDur)[0]?.run_id ?? "";
    onChange({ stackLabel: newStack, duration: newDur, runId: newRunId });
  }

  function handleDurationChange(newDur) {
    const newRunId = allRuns.filter((r) => r.duration === newDur)[0]?.run_id ?? "";
    onChange({ stackLabel, duration: newDur, runId: newRunId });
  }

  function handleRunChange(newRunId) {
    onChange({ stackLabel, duration, runId: newRunId });
  }

  return (
    <div className="run-selector-row">
      <div className="run-selector-dot" style={{ background: seriesColor(index) }} />
      <span className="run-selector-label">#{index + 1}</span>

      {/* Stack */}
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

      {/* Run time (duration) */}
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

      {/* Specific run */}
      <div className="control-group" style={{ margin: 0 }}>
        <select
          className="control-select"
          style={{ minWidth: 230 }}
          value={runId}
          onChange={(e) => handleRunChange(e.target.value)}
          disabled={!runsForDuration.length}
        >
          {!runsForDuration.length && <option value="">—</option>}
          {runsForDuration.map((r) => (
            <option key={r.run_id} value={r.run_id}>{runLabel(r)}</option>
          ))}
        </select>
      </div>

      <button className="run-selector-remove" onClick={onRemove} title="Remove">×</button>
    </div>
  );
}

export default function Compare() {
  const [stacks, setStacks] = useState({});
  const [stackKeys, setStackKeys] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  // selections: [{ stackLabel, duration, runId }]
  const [selections, setSelections] = useState([
    { stackLabel: "", duration: "", runId: "" },
    { stackLabel: "", duration: "", runId: "" },
  ]);

  const [runDetails, setRunDetails] = useState({});

  useEffect(() => {
    loadIndex()
      .then((idx) => {
        const grouped = groupByStack(idx.runs);
        const keys = Object.keys(grouped).sort();
        setStacks(grouped);
        setStackKeys(keys);
        // Pre-fill first two rows with the first two distinct stacks
        setSelections((prev) =>
          prev.map((sel, i) => {
            if (sel.stackLabel || !keys[i]) return sel;
            const sl = keys[i];
            const runs = grouped[sl] || [];
            const durs = durationsForRuns(runs);
            const dur = durs[0] ?? "";
            const runId = runs.filter((r) => r.duration === dur)[0]?.run_id ?? "";
            return { stackLabel: sl, duration: dur, runId };
          })
        );
        setLoading(false);
      })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  // Load detail JSON for any newly selected run
  useEffect(() => {
    for (const { runId } of selections) {
      if (runId && !runDetails[runId]) {
        loadRun(runId)
          .then((r) => setRunDetails((prev) => ({ ...prev, [runId]: r })))
          .catch(() => {});
      }
    }
  }, [selections]);

  const updateSelection = (i, val) =>
    setSelections((prev) => prev.map((s, idx) => (idx === i ? val : s)));
  const removeSelection = (i) =>
    setSelections((prev) => prev.filter((_, idx) => idx !== i));
  const addSelection = () =>
    setSelections((prev) => [...prev, { stackLabel: "", duration: "", runId: "" }]);

  if (loading) return <div className="loading">Loading index…</div>;
  if (error) return <div className="loading" style={{ color: "#ef4444" }}>Error: {error}</div>;

  // Build labelled series for charts
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
      {/* Selector rows */}
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
            disabled={selections.length >= MAX_SERIES}
          >
            + Add run
          </button>
        </div>
      </div>

      {!hasSeries && (
        <div className="empty-state">
          <div className="empty-state-icon">🔀</div>
          <div>Select at least one stack and run above to start comparing.</div>
        </div>
      )}

      {hasSeries && (
        <>
          {/* Color legend */}
          <div style={{ display: "flex", gap: 16, marginBottom: 20, flexWrap: "wrap" }}>
            {labelledSeries.map((s) => (
              <div key={s.label} style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
                <span style={{ width: 12, height: 12, borderRadius: "50%", background: s.color, display: "inline-block" }} />
                <span style={{ color: "#e2e8f0" }}>{s.label}</span>
              </div>
            ))}
          </div>

          {/* Comparison charts */}
          <div className="charts-grid">
            <RpsCompareChart series={labelledSeries} />
            <ErrorRateCompareChart series={labelledSeries} />
            <LatencyCompareChart series={labelledSeries} />
            <CpuCompareChart series={labelledSeries} />
            <RamCompareChart series={labelledSeries} />
            <PgCompareChart series={labelledSeries} />
          </div>

          {/* Time-series overlays */}
          {cpuOverlaySeries.some((s) => s.data.length > 0) && (
            <div style={{ marginTop: 20, display: "grid", gap: 20 }}>
              <CpuOverlayChart series={cpuOverlaySeries} />
              <RamOverlayChart series={ramOverlaySeries} />
            </div>
          )}
        </>
      )}
    </div>
  );
}
