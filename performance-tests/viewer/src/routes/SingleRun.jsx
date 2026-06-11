import React, { useEffect, useState } from "react";
import { loadIndex, loadRun, groupByStack, durationsForRuns, runLabel } from "../lib/data.js";
import StatCards from "../components/StatCards.jsx";
import LatencyChart from "../components/LatencyChart.jsx";
import { CpuTimeSeriesChart, RamTimeSeriesChart } from "../components/TimeSeriesChart.jsx";
import EndpointChecksChart from "../components/EndpointChecksChart.jsx";
import PgCountersChart from "../components/PgCountersChart.jsx";

export default function SingleRun() {
  const [stacks, setStacks] = useState({});   // label -> runs[]
  const [stackKeys, setStackKeys] = useState([]);
  const [selectedStack, setSelectedStack] = useState("");
  const [selectedDuration, setSelectedDuration] = useState("");
  const [selectedRunId, setSelectedRunId] = useState("");
  const [run, setRun] = useState(null);
  const [loading, setLoading] = useState(true);
  const [runLoading, setRunLoading] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadIndex()
      .then((idx) => {
        const grouped = groupByStack(idx.runs);
        const keys = Object.keys(grouped).sort();
        setStacks(grouped);
        setStackKeys(keys);
        if (keys.length) setSelectedStack(keys[0]);
        setLoading(false);
      })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  // When stack changes, reset duration and pick first available
  useEffect(() => {
    if (!selectedStack || !stacks[selectedStack]?.length) {
      setSelectedDuration("");
      setSelectedRunId("");
      setRun(null);
      return;
    }
    const durations = durationsForRuns(stacks[selectedStack]);
    setSelectedDuration(durations[0] ?? "");
  }, [selectedStack, stacks]);

  // When duration changes, pick first matching run
  useEffect(() => {
    if (!selectedStack || !selectedDuration) { setSelectedRunId(""); setRun(null); return; }
    const filtered = (stacks[selectedStack] || []).filter((r) => r.duration === selectedDuration);
    setSelectedRunId(filtered[0]?.run_id ?? "");
  }, [selectedDuration, selectedStack, stacks]);

  // When run id changes, load detail
  useEffect(() => {
    if (!selectedRunId) { setRun(null); return; }
    setRunLoading(true);
    loadRun(selectedRunId)
      .then((r) => { setRun(r); setRunLoading(false); })
      .catch((e) => { setError(e.message); setRunLoading(false); });
  }, [selectedRunId]);

  if (loading) return <div className="loading">Loading index…</div>;
  if (error) return <div className="loading" style={{ color: "#ef4444" }}>Error: {error}</div>;
  if (!stackKeys.length) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">📂</div>
        <div>No benchmark results found.</div>
        <div style={{ fontSize: 13, color: "#8892a4" }}>Run <code>./run.sh &lt;stack&gt; naive</code> from <code>performance-tests/</code> first.</div>
      </div>
    );
  }

  const runsForStack = stacks[selectedStack] || [];
  const durations = durationsForRuns(runsForStack);
  const runsForDuration = runsForStack.filter((r) => r.duration === selectedDuration);

  return (
    <div>
      {/* Three-step selectors */}
      <div className="controls">
        <div className="control-group">
          <label className="control-label">Stack</label>
          <select
            className="control-select"
            value={selectedStack}
            onChange={(e) => setSelectedStack(e.target.value)}
          >
            {stackKeys.map((k) => <option key={k} value={k}>{k}</option>)}
          </select>
        </div>

        <div className="control-group">
          <label className="control-label">Run time</label>
          <select
            className="control-select"
            value={selectedDuration}
            onChange={(e) => setSelectedDuration(e.target.value)}
            disabled={!durations.length}
          >
            {!durations.length && <option value="">—</option>}
            {durations.map((d) => <option key={d} value={d}>{d}</option>)}
          </select>
        </div>

        <div className="control-group">
          <label className="control-label">Run</label>
          <select
            className="control-select"
            value={selectedRunId}
            onChange={(e) => setSelectedRunId(e.target.value)}
            disabled={!runsForDuration.length}
          >
            {!runsForDuration.length && <option value="">—</option>}
            {runsForDuration.map((r) => (
              <option key={r.run_id} value={r.run_id}>{runLabel(r)}</option>
            ))}
          </select>
        </div>
      </div>

      {runLoading && <div className="loading">Loading run data…</div>}

      {run && !runLoading && (
        <>
          {/* Meta badges */}
          <div className="run-meta-badges">
            <span className="badge"><strong>Stack</strong> {run.meta.label}</span>
            <span className="badge"><strong>Variant</strong> {run.meta.variant}</span>
            <span className="badge"><strong>VUs</strong> {run.meta.vus ?? "—"}</span>
            <span className="badge"><strong>Duration</strong> {run.meta.duration || "—"}</span>
            <span className="badge"><strong>CPUs</strong> {run.meta.app_cpus ?? "—"}</span>
            <span className="badge"><strong>Memory</strong> {run.meta.app_memory || "—"}</span>
            <span className="badge"><strong>Timestamp</strong> {run.meta.timestamp}</span>
          </div>

          {/* Stat cards */}
          <StatCards run={run} />

          {/* Charts */}
          <div className="charts-grid">
            <LatencyChart k6={run.k6} />

            <div className="chart-card">
              <div className="chart-title">Resource Summary</div>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 13 }}>
                <tbody>
                  {[
                    ["CPU avg", run.resources.cpu_avg_pct, "%"],
                    ["CPU peak", run.resources.cpu_peak_pct, "%"],
                    ["RAM avg", run.resources.mem_avg_mb, "MB"],
                    ["RAM peak", run.resources.mem_peak_mb, "MB"],
                    ["Net RX total", run.resources.net_rx_total_mb, "MB"],
                    ["Net TX total", run.resources.net_tx_total_mb, "MB"],
                    ["Blk read total", run.resources.blk_read_total_mb, "MB"],
                    ["Blk write total", run.resources.blk_write_total_mb, "MB"],
                  ].map(([label, val, unit]) => (
                    <tr key={label} style={{ borderBottom: "1px solid #2e3347" }}>
                      <td style={{ padding: "7px 0", color: "#8892a4" }}>{label}</td>
                      <td style={{ padding: "7px 0", textAlign: "right", color: "#e2e8f0", fontWeight: 600 }}>
                        {val != null ? `${val.toFixed(1)} ${unit}` : "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <CpuTimeSeriesChart timeseries={run.timeseries} />
            <RamTimeSeriesChart timeseries={run.timeseries} />
            <EndpointChecksChart endpoints={run.endpoints} />
            <PgCountersChart pg={run.pg} />
          </div>
        </>
      )}
    </div>
  );
}
