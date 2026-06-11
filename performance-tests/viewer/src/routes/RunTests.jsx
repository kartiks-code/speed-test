import React, { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { fetchStacks, enqueueRun } from "../lib/api.js";

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
      {/* Visual bar */}
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

export default function RunTests() {
  const navigate = useNavigate();
  const [stacks, setStacks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [submitting, setSubmitting] = useState(false);
  const [successMsg, setSuccessMsg] = useState(null);

  // form state
  const [selectedStack, setSelectedStack] = useState("");
  const [selectedVariant, setSelectedVariant] = useState("");
  const [durationSec, setDurationSec] = useState(60);
  const [vus, setVus] = useState(20);
  const [mix, setMix] = useState({ ...DEFAULT_MIX });

  useEffect(() => {
    fetchStacks()
      .then((data) => {
        setStacks(data);
        if (data.length > 0) {
          setSelectedStack(data[0].id);
          if (data[0].variants.length > 0) setSelectedVariant(data[0].variants[0].label);
        }
        setLoading(false);
      })
      .catch((e) => { setError(e.message); setLoading(false); });
  }, []);

  // When stack changes, reset variant to first available
  useEffect(() => {
    if (!selectedStack) return;
    const stack = stacks.find((s) => s.id === selectedStack);
    if (stack?.variants.length > 0) {
      setSelectedVariant(stack.variants[0].label);
    } else {
      setSelectedVariant("");
    }
  }, [selectedStack, stacks]);

  const currentStack = stacks.find((s) => s.id === selectedStack);
  const currentVariant = currentStack?.variants.find((v) => v.label === selectedVariant);

  const handleSubmit = useCallback(
    async (e) => {
      e.preventDefault();
      if (!selectedStack || !selectedVariant) return;
      setSubmitting(true);
      setSuccessMsg(null);
      setError(null);
      try {
        const params = {
          stackId: selectedStack,
          variant: selectedVariant,
          durationSec: Number(durationSec),
          vus: Number(vus),
          mix: {
            create: mix.create,
            read:   mix.read,
            update: mix.update,
            delete: mix.delete,
          },
        };
        if (currentVariant && currentVariant.dockerfile !== "Dockerfile" && currentVariant.dockerfile !== "Dockerfile.optimized") {
          params.dockerfileOverride = currentVariant.dockerfile;
        }
        await enqueueRun(params);
        setSuccessMsg("Added to queue. Tests run sequentially.");
        setTimeout(() => navigate("/queue"), 800);
      } catch (err) {
        setError(err.message);
      } finally {
        setSubmitting(false);
      }
    },
    [selectedStack, selectedVariant, durationSec, vus, mix, currentVariant, navigate]
  );

  if (loading) return <div className="loading">Loading stacks…</div>;

  return (
    <div className="run-tests-page">
      <h2 className="page-title">Run a Benchmark</h2>
      {error && <div className="error-banner">{error}</div>}
      {successMsg && <div className="success-banner">{successMsg}</div>}

      <form className="run-form" onSubmit={handleSubmit}>
        {/* Stack + Variant */}
        <div className="form-section">
          <div className="form-section-title">Target</div>
          <div className="form-row">
            <div className="control-group">
              <label className="control-label">Stack</label>
              <select
                className="control-select"
                value={selectedStack}
                onChange={(e) => setSelectedStack(e.target.value)}
              >
                {stacks.map((s) => (
                  <option key={s.id} value={s.id}>{s.label}</option>
                ))}
              </select>
              {currentStack?.notes && (
                <div className="field-note">{currentStack.notes}</div>
              )}
            </div>
            <div className="control-group">
              <label className="control-label">Variant</label>
              <select
                className="control-select"
                value={selectedVariant}
                onChange={(e) => setSelectedVariant(e.target.value)}
                disabled={!currentStack?.variants.length}
              >
                {(currentStack?.variants ?? []).map((v) => (
                  <option key={v.label} value={v.label}>
                    {v.label} ({v.dockerfile})
                  </option>
                ))}
              </select>
            </div>
          </div>
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
            disabled={submitting || !selectedStack || !selectedVariant}
          >
            {submitting ? "Adding…" : "Add to Queue"}
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
