import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  loadIndex,
  groupBySuite,
  compareRunLabel,
  mixPercentages,
} from "../lib/data.js";
import {
  filterRuns,
  buildFilterOptions,
  formatRunTimestamp,
  dateInputToRunTimestamp,
  timestampAfterForPreset,
  TIME_PRESETS,
} from "../lib/runFilters.js";
import {
  assignRunsToSuite,
  deleteRuns,
  dissolveSuite,
  deleteSuiteRuns,
  pingServer,
} from "../lib/api.js";
import { useDataRefresh } from "../lib/DataRefreshContext.jsx";
import CheckboxGroup from "../components/CheckboxGroup.jsx";

function MixSummary({ run }) {
  const pct = mixPercentages(run.mix);
  if (!pct) return <span className="manage-run-mix">Full CRUD</span>;
  return (
    <span className="manage-run-mix">
      C{pct.create} R{pct.read} U{pct.update} D{pct.delete}
    </span>
  );
}

export default function ManageRuns() {
  const { refreshToken } = useDataRefresh();
  const [allRuns, setAllRuns] = useState([]);
  const [suites, setSuites] = useState({});
  const [filterOptions, setFilterOptions] = useState({
    suiteNames: [],
    stacks: [],
    durations: [],
    variants: [],
    vus: [],
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [serverOk, setServerOk] = useState(null);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState(null);
  const [actionMessage, setActionMessage] = useState(null);

  const [suiteQuery, setSuiteQuery] = useState("");
  const [selectedStacks, setSelectedStacks] = useState([]);
  const [selectedDurations, setSelectedDurations] = useState([]);
  const [selectedVariants, setSelectedVariants] = useState([]);
  const [selectedVus, setSelectedVus] = useState([]);
  const [timePreset, setTimePreset] = useState("all");
  const [timestampAfterInput, setTimestampAfterInput] = useState("");
  const [timestampBeforeInput, setTimestampBeforeInput] = useState("");
  const [unsuitedOnly, setUnsuitedOnly] = useState(false);

  const [selectedRunIds, setSelectedRunIds] = useState([]);
  const [newSuiteName, setNewSuiteName] = useState("");

  const reloadIndex = useCallback(() => {
    setLoading(true);
    setError(null);
    return loadIndex({ force: true })
      .then((idx) => {
        setAllRuns(idx.runs);
        setSuites(groupBySuite(idx.runs));
        setFilterOptions(buildFilterOptions(idx.runs));
        setLoading(false);
      })
      .catch((e) => {
        setError(e.message);
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    reloadIndex();
    pingServer().then(setServerOk);
  }, [reloadIndex, refreshToken]);

  const timestampAfter = useMemo(() => {
    const custom = dateInputToRunTimestamp(timestampAfterInput);
    if (custom) return custom;
    return timestampAfterForPreset(timePreset);
  }, [timestampAfterInput, timePreset]);

  const timestampBefore = useMemo(
    () => dateInputToRunTimestamp(timestampBeforeInput),
    [timestampBeforeInput]
  );

  const filteredRuns = useMemo(
    () =>
      filterRuns(allRuns, {
        suiteQuery,
        stacks: selectedStacks,
        durations: selectedDurations,
        variants: selectedVariants,
        vus: selectedVus,
        timestampAfter,
        timestampBefore,
        unsuitedOnly,
      }),
    [
      allRuns,
      suiteQuery,
      selectedStacks,
      selectedDurations,
      selectedVariants,
      selectedVus,
      timestampAfter,
      timestampBefore,
      unsuitedOnly,
    ]
  );

  const filteredRunIds = useMemo(
    () => filteredRuns.map((r) => r.run_id),
    [filteredRuns]
  );

  const allFilteredSelected =
    filteredRunIds.length > 0 &&
    filteredRunIds.every((id) => selectedRunIds.includes(id));

  function toggleRunSelection(runId) {
    setSelectedRunIds((prev) =>
      prev.includes(runId) ? prev.filter((id) => id !== runId) : [...prev, runId]
    );
  }

  function selectAllFiltered() {
    setSelectedRunIds((prev) => [...new Set([...prev, ...filteredRunIds])]);
  }

  function clearSelection() {
    setSelectedRunIds([]);
  }

  function selectFilteredOnly() {
    setSelectedRunIds(filteredRunIds);
  }

  async function runAction(fn) {
    setActionError(null);
    setActionMessage(null);
    setBusy(true);
    try {
      const result = await fn();
      setActionMessage(typeof result === "string" ? result : "Done.");
      setSelectedRunIds([]);
      await reloadIndex();
    } catch (e) {
      setActionError(e.message);
    } finally {
      setBusy(false);
    }
  }

  function handleAssignSuite() {
    const name = newSuiteName.trim();
    if (!name) {
      setActionError("Enter a suite name.");
      return;
    }
    if (!selectedRunIds.length) {
      setActionError("Select at least one run.");
      return;
    }
    runAction(async () => {
      const result = await assignRunsToSuite({ runIds: selectedRunIds, suiteName: name });
      setNewSuiteName("");
      return `Assigned ${result.count} run(s) to suite "${result.suiteName}".`;
    });
  }

  function handleDeleteSelected() {
    if (!selectedRunIds.length) {
      setActionError("Select at least one run.");
      return;
    }
    const n = selectedRunIds.length;
    if (!window.confirm(`Permanently delete ${n} run result${n !== 1 ? "s" : ""}? This cannot be undone.`)) {
      return;
    }
    runAction(async () => {
      const result = await deleteRuns({ runIds: selectedRunIds });
      return `Deleted ${result.deleted} run(s).`;
    });
  }

  function handleDissolveSuite(name) {
    const count = suites[name]?.length ?? 0;
    if (!window.confirm(`Remove suite label from ${count} run(s) in "${name}"? Runs will remain on disk.`)) {
      return;
    }
    runAction(async () => {
      const result = await dissolveSuite(name);
      return `Removed suite label from ${result.count} run(s).`;
    });
  }

  function handleDeleteSuiteRuns(name) {
    const count = suites[name]?.length ?? 0;
    if (!window.confirm(`Permanently delete all ${count} run(s) in suite "${name}"? This cannot be undone.`)) {
      return;
    }
    runAction(async () => {
      const result = await deleteSuiteRuns(name);
      return `Deleted ${result.deleted} run(s) from suite "${name}".`;
    });
  }

  function loadSuiteFilter(name) {
    setSuiteQuery(name);
    setUnsuitedOnly(false);
    const ids = filterRuns(allRuns, {
      suiteQuery: name,
      stacks: selectedStacks,
      durations: selectedDurations,
      variants: selectedVariants,
      vus: selectedVus,
      timestampAfter,
      timestampBefore,
      unsuitedOnly: false,
    }).map((r) => r.run_id);
    setSelectedRunIds(ids);
  }

  const suiteNameList = filterOptions.suiteNames;
  const filteredSuiteNames = suiteNameList.filter((n) =>
    !suiteQuery.trim() || n.toLowerCase().includes(suiteQuery.trim().toLowerCase())
  );

  if (loading) return <div className="loading">Loading runs…</div>;
  if (error) return <div className="loading" style={{ color: "#ef4444" }}>Error: {error}</div>;

  return (
    <div className="manage-runs">
      <div className="manage-runs-header">
        <div>
          <h2 className="page-title">Manage Runs</h2>
          <p className="page-subtitle">
            Filter runs like Compare, select multiple, assign suite names after the fact, or delete runs and suites.
          </p>
        </div>
        {serverOk === false && (
          <div className="manage-server-warning">
            Control server offline — start with{" "}
            <code>npm run server</code> or <code>npm run dev:all</code> to assign suites or delete runs.
          </div>
        )}
      </div>

      {actionError && <div className="manage-action-error">{actionError}</div>}
      {actionMessage && <div className="manage-action-success">{actionMessage}</div>}

      <div className="manage-layout">
        <aside className="manage-filters">
          <div className="control-group">
            <label className="control-label">Search suite</label>
            <input
              type="search"
              className="control-input"
              placeholder="Filter by suite name…"
              value={suiteQuery}
              onChange={(e) => setSuiteQuery(e.target.value)}
              list="manage-suite-suggestions"
            />
            <datalist id="manage-suite-suggestions">
              {suiteNameList.map((n) => <option key={n} value={n} />)}
            </datalist>
          </div>

          <label className="manage-checkbox-inline">
            <input
              type="checkbox"
              checked={unsuitedOnly}
              onChange={(e) => setUnsuitedOnly(e.target.checked)}
            />
            <span>Only runs without a suite</span>
          </label>

          {filterOptions.stacks.length > 0 && (
            <CheckboxGroup
              label="Stack"
              options={filterOptions.stacks.map((s) => ({ id: s, label: s }))}
              selected={selectedStacks}
              onChange={setSelectedStacks}
            />
          )}

          {filterOptions.durations.length > 0 && (
            <CheckboxGroup
              label="Duration"
              options={filterOptions.durations.map((d) => ({ id: d, label: d }))}
              selected={selectedDurations}
              onChange={setSelectedDurations}
            />
          )}

          {filterOptions.variants.length > 0 && (
            <CheckboxGroup
              label="Variant"
              options={filterOptions.variants.map((v) => ({ id: v, label: v }))}
              selected={selectedVariants}
              onChange={setSelectedVariants}
            />
          )}

          {filterOptions.vus.length > 0 && (
            <CheckboxGroup
              label="VUs"
              options={filterOptions.vus.map((v) => ({ id: v, label: String(v) }))}
              selected={selectedVus}
              onChange={setSelectedVus}
            />
          )}

          <div className="control-group">
            <label className="control-label">Run time</label>
            <select
              className="control-select"
              value={timePreset}
              onChange={(e) => setTimePreset(e.target.value)}
            >
              {TIME_PRESETS.map((p) => (
                <option key={p.id} value={p.id}>{p.label}</option>
              ))}
            </select>
          </div>

          <div className="manage-time-range">
            <div className="control-group">
              <label className="control-label">After (UTC)</label>
              <input
                type="datetime-local"
                className="control-input"
                value={timestampAfterInput}
                onChange={(e) => {
                  setTimestampAfterInput(e.target.value);
                  if (e.target.value) setTimePreset("all");
                }}
              />
            </div>
            <div className="control-group">
              <label className="control-label">Before (UTC)</label>
              <input
                type="datetime-local"
                className="control-input"
                value={timestampBeforeInput}
                onChange={(e) => setTimestampBeforeInput(e.target.value)}
              />
            </div>
          </div>

          <div className="run-count-summary">
            <strong>{filteredRuns.length}</strong> of {allRuns.length} run(s) match filters
          </div>
        </aside>

        <div className="manage-main">
          {selectedRunIds.length > 0 && (
            <div className="manage-bulk-bar">
              <span className="manage-bulk-count">{selectedRunIds.length} selected</span>
              <div className="manage-bulk-actions">
                <input
                  type="text"
                  className="control-input manage-suite-input"
                  placeholder="Suite name…"
                  value={newSuiteName}
                  onChange={(e) => setNewSuiteName(e.target.value)}
                  disabled={!serverOk || busy}
                />
                <button
                  type="button"
                  className="btn-primary btn-sm"
                  onClick={handleAssignSuite}
                  disabled={!serverOk || busy}
                >
                  Name as suite
                </button>
                <button
                  type="button"
                  className="btn-cancel btn-sm"
                  onClick={handleDeleteSelected}
                  disabled={!serverOk || busy}
                >
                  Delete selected
                </button>
                <button type="button" className="btn-link" onClick={clearSelection} disabled={busy}>
                  Clear selection
                </button>
              </div>
            </div>
          )}

          <div className="manage-table-toolbar">
            <button type="button" className="btn-secondary btn-sm" onClick={selectAllFiltered}>
              Select all filtered ({filteredRuns.length})
            </button>
            <button
              type="button"
              className="btn-secondary btn-sm"
              onClick={selectFilteredOnly}
            >
              Select filtered only
            </button>
            <button type="button" className="btn-link" onClick={clearSelection} disabled={!selectedRunIds.length}>
              Clear
            </button>
          </div>

          <div className="manage-table-wrap">
            <table className="manage-table">
              <thead>
                <tr>
                  <th className="manage-col-check">
                    <input
                      type="checkbox"
                      checked={allFilteredSelected}
                      onChange={() => (allFilteredSelected ? clearSelection() : selectAllFiltered())}
                      title="Toggle all filtered"
                    />
                  </th>
                  <th>Stack</th>
                  <th>Variant</th>
                  <th>Duration</th>
                  <th>VUs</th>
                  <th>Mix</th>
                  <th>Run time</th>
                  <th>Suite</th>
                </tr>
              </thead>
              <tbody>
                {filteredRuns.length === 0 ? (
                  <tr>
                    <td colSpan={8} className="manage-empty-row">No runs match the current filters.</td>
                  </tr>
                ) : (
                  filteredRuns.map((run) => (
                    <tr
                      key={run.run_id}
                      className={selectedRunIds.includes(run.run_id) ? "manage-row-selected" : ""}
                    >
                      <td className="manage-col-check">
                        <input
                          type="checkbox"
                          checked={selectedRunIds.includes(run.run_id)}
                          onChange={() => toggleRunSelection(run.run_id)}
                        />
                      </td>
                      <td>{run.label}</td>
                      <td><span className="badge badge-sm">{run.variant}</span></td>
                      <td>{run.duration}</td>
                      <td>{run.vus ?? "—"}</td>
                      <td><MixSummary run={run} /></td>
                      <td className="manage-ts" title={run.run_id}>
                        {compareRunLabel(run)}
                        <span className="manage-ts-full">{formatRunTimestamp(run.timestamp)}</span>
                      </td>
                      <td>
                        {run.suite ? (
                          <button
                            type="button"
                            className="suite-suggestion-btn manage-suite-link"
                            onClick={() => loadSuiteFilter(run.suite)}
                          >
                            {run.suite}
                          </button>
                        ) : (
                          <span className="manage-no-suite">—</span>
                        )}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          {suiteNameList.length > 0 && (
            <section className="manage-suites-section">
              <h3 className="manage-section-title">Suites</h3>
              <p className="manage-section-desc">
                Dissolve removes the suite label from runs. Delete removes all run data for that suite.
              </p>
              <div className="manage-suite-list">
                {(suiteQuery.trim() ? filteredSuiteNames : suiteNameList).map((name) => (
                  <div key={name} className="manage-suite-row">
                    <div className="manage-suite-info">
                      <button
                        type="button"
                        className="btn-link manage-suite-name"
                        onClick={() => loadSuiteFilter(name)}
                      >
                        {name}
                      </button>
                      <span className="suite-suggestion-count">{suites[name]?.length ?? 0} runs</span>
                      <Link to={`/compare?suite=${encodeURIComponent(name)}`} className="btn-link manage-compare-link">
                        Compare →
                      </Link>
                    </div>
                    <div className="manage-suite-actions">
                      <button
                        type="button"
                        className="btn-secondary btn-sm"
                        onClick={() => handleDissolveSuite(name)}
                        disabled={!serverOk || busy}
                      >
                        Dissolve
                      </button>
                      <button
                        type="button"
                        className="btn-cancel btn-sm"
                        onClick={() => handleDeleteSuiteRuns(name)}
                        disabled={!serverOk || busy}
                      >
                        Delete runs
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
