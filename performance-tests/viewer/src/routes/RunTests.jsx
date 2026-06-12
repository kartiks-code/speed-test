import React, { useEffect, useState, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { fetchStacks, enqueueSuite } from "../lib/api.js";
import CheckboxGroup from "../components/CheckboxGroup.jsx";

const DEFAULT_MIX = { create: 25, read: 25, update: 25, delete: 25 };
const MIX_COLORS = {
  create: "#22c55e",
  update: "#6366f1",
  read:   "#f59e0b",
  delete: "#ef4444",
};
const MIX_LABELS = { create: "Create", read: "Read", update: "Update", delete: "Delete" };

function normalizeMix(mix) {
  const total = mix.create + mix.read + mix.update + mix.delete;
  if (total === 0) return { create: 25, read: 25, update: 25, delete: 25 };
  return {
    create: Math.round((mix.create / total) * 100),
    read:   Math.round((mix.read   / total) * 100),
    update: Math.round((mix.update / total) * 100),
    delete: Math.round((mix.delete / total) * 100),
  };
}

function MixSliders({ mix, onChange }) {
  const pct = normalizeMix(mix);
  const total = mix.create + mix.read + mix.update + mix.delete;

  return (
    <div className="mix-sliders">
      {["create", "read", "update", "delete"].map((op) => (
        <div key={op} className="mix-row">
          <label className="mix-label" style={{ color: MIX_COLORS[op] }}>
            {MIX_LABELS[op]}
          </label>
          <input
            type="range"
            min={0}
            max={100}
            value={mix[op]}
            onChange={(e) => onChange({ ...mix, [op]: parseInt(e.target.value, 10) })}
            className="mix-range"
            style={{ "--mix-color": MIX_COLORS[op] }}
          />
          <span className="mix-weight">{mix[op]}</span>
          <span className="mix-pct" style={{ color: MIX_COLORS[op] }}>
            {total > 0 ? `${pct[op]}%` : "—"}
          </span>
        </div>
      ))}
      {total > 0 && (
        <div className="mix-bar">
          {["create", "read", "update", "delete"].map((op) =>
            pct[op] > 0 ? (
              <div
                key={op}
                title={`${MIX_LABELS[op]} ${pct[op]}%`}
                style={{ width: `${pct[op]}%`, background: MIX_COLORS[op] }}
              />
            ) : null
          )}
        </div>
      )}
    </div>
  );
}

const STANDARD_VARIANTS = [
  { id: "naive", label: "naive", hint: "Dockerfile" },
  { id: "optimized", label: "optimized", hint: "Dockerfile.optimized" },
];

/** Variant options: always show naive/optimized; add custom variants from selected stacks. */
function variantOptionsForStacks(stacks, selectedStackIds) {
  const seen = new Map(STANDARD_VARIANTS.map((v) => [v.id, v.hint]));

  for (const stack of stacks) {
    if (selectedStackIds.length > 0 && !selectedStackIds.includes(stack.id)) continue;
    for (const v of stack.variants) {
      if (!seen.has(v.label)) {
        seen.set(v.label, v.dockerfile);
      }
    }
  }

  return [...seen.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([label, dockerfile]) => ({
      id: label,
      label,
      hint: dockerfile,
    }));
}

export default function RunTests() {
  const navigate = useNavigate();
  const [stacks, setStacks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [submitting, setSubmitting] = useState(false);
  const [successMsg, setSuccessMsg] = useState(null);

  const [suiteName, setSuiteName] = useState("");
  const [selectedStacks, setSelectedStacks] = useState([]);
  const [selectedVariants, setSelectedVariants] = useState(["naive", "optimized"]);
  const [durationSec, setDurationSec] = useState(60);
  const [vus, setVus] = useState(20);
  const [mix, setMix] = useState({ ...DEFAULT_MIX });

  useEffect(() => {
    fetchStacks()
      .then((data) => {
        setStacks(data);
        setLoading(false);
      })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  const stackOptions = useMemo(
    () => stacks.map((s) => ({ id: s.id, label: s.label })),
    [stacks]
  );

  const variantOptions = useMemo(
    () => variantOptionsForStacks(stacks, selectedStacks),
    [stacks, selectedStacks]
  );

  // Drop variant selections that are no longer available for the chosen stacks
  useEffect(() => {
    const available = new Set(variantOptions.map((o) => o.id));
    setSelectedVariants((prev) => prev.filter((v) => available.has(v)));
  }, [variantOptions]);

  const runCount = selectedStacks.length * selectedVariants.length;
  const canSubmit =
    suiteName.trim().length > 0 &&
    selectedStacks.length > 0 &&
    selectedVariants.length > 0;

  const handleSubmit = useCallback(
    async (e) => {
      e.preventDefault();
      if (!canSubmit) return;
      setSubmitting(true);
      setSuccessMsg(null);
      setError(null);
      try {
        const result = await enqueueSuite({
          suiteName: suiteName.trim(),
          stackIds: selectedStacks,
          variants: selectedVariants,
          durationSec: Number(durationSec),
          vus: Number(vus),
          mix: {
            create: mix.create,
            read:   mix.read,
            update: mix.update,
            delete: mix.delete,
          },
        });
        setSuccessMsg(
          `Scheduled ${result.count} run(s) in suite "${result.suiteName}". Tests run sequentially.`
        );
        setTimeout(() => navigate("/queue"), 800);
      } catch (err) {
        setError(err.message);
      } finally {
        setSubmitting(false);
      }
    },
    [canSubmit, suiteName, selectedStacks, selectedVariants, durationSec, vus, mix, navigate]
  );

  if (loading) return <div className="loading">Loading stacks…</div>;

  return (
    <div className="run-tests-page">
      <h2 className="page-title">Run a Benchmark Suite</h2>
      {error && <div className="error-banner">{error}</div>}
      {successMsg && <div className="success-banner">{successMsg}</div>}

      <form className="run-form" onSubmit={handleSubmit}>
        {/* Suite name */}
        <div className="form-section">
          <div className="form-section-title">Suite</div>
          <div className="control-group">
            <label className="control-label">Suite name</label>
            <input
              type="text"
              className="control-input"
              placeholder="e.g. jvm-comparison-june"
              value={suiteName}
              onChange={(e) => setSuiteName(e.target.value)}
              maxLength={120}
            />
            <div className="field-note">
              All runs in this batch share this name. Search by suite in Single Run and Compare.
            </div>
          </div>
        </div>

        {/* Stack + Variant multi-select */}
        <div className="form-section">
          <div className="form-section-title">Targets</div>
          <div className="form-row form-row--stacks">
            <CheckboxGroup
              label="Stacks"
              options={stackOptions}
              selected={selectedStacks}
              onChange={setSelectedStacks}
            />
            <CheckboxGroup
              label="Variants"
              options={variantOptions}
              selected={selectedVariants}
              onChange={setSelectedVariants}
            />
          </div>
          {runCount > 0 && (
            <div className="run-count-summary">
              <strong>{runCount}</strong> run{runCount !== 1 ? "s" : ""} will be scheduled
              ({selectedStacks.length} stack{selectedStacks.length !== 1 ? "s" : ""} ×{" "}
              {selectedVariants.length} variant{selectedVariants.length !== 1 ? "s" : ""})
            </div>
          )}
        </div>

        {/* Duration + VUs */}
        <div className="form-section">
          <div className="form-section-title">Load</div>
          <div className="form-row">
            <div className="control-group">
              <label className="control-label">Duration (seconds)</label>
              <div className="input-with-hint">
                <input
                  type="number"
                  className="control-input"
                  min={5}
                  max={3600}
                  value={durationSec}
                  onChange={(e) => setDurationSec(e.target.value)}
                />
                <span className="input-hint">{durationSec}s = {(durationSec / 60).toFixed(1)} min</span>
              </div>
            </div>
            <div className="control-group">
              <label className="control-label">Virtual Users (VUs)</label>
              <input
                type="number"
                className="control-input"
                min={1}
                max={500}
                value={vus}
                onChange={(e) => setVus(e.target.value)}
              />
            </div>
          </div>
        </div>

        {/* CRUD Mix */}
        <div className="form-section">
          <div className="form-section-title">
            Operation Mix
            <button
              type="button"
              className="btn-link"
              onClick={() => setMix({ ...DEFAULT_MIX })}
            >
              reset to equal
            </button>
          </div>
          <p className="field-note" style={{ marginBottom: 12 }}>
            Drag sliders to set relative weights. Each VU iteration performs one
            operation drawn by these proportions. Zero on all defaults to 25/25/25/25.
          </p>
          <MixSliders mix={mix} onChange={setMix} />
        </div>

        <div className="form-actions">
          <button
            type="submit"
            className="btn-primary"
            disabled={submitting || !canSubmit}
          >
            {submitting ? "Adding…" : `Add ${runCount || ""} run${runCount !== 1 ? "s" : ""} to Queue`}
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={() => navigate("/queue")}
          >
            View Queue
          </button>
        </div>
      </form>
    </div>
  );
}
