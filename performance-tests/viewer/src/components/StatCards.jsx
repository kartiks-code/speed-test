import React from "react";
import { fmtRps, fmtMs, fmtPct, fmt } from "../lib/data.js";

function StatCard({ label, value, unit, tone }) {
  return (
    <div className={`stat-card ${tone || ""}`}>
      <div className="stat-card-label">{label}</div>
      <div className="stat-card-value">
        {value}
        {unit && <span className="stat-card-unit">{unit}</span>}
      </div>
    </div>
  );
}

function errorTone(rate) {
  if (rate == null) return "";
  if (rate < 0.01) return "good";
  if (rate < 0.05) return "warn";
  return "danger";
}

export default function StatCards({ run }) {
  const r = run.k6;
  const res = run.resources;

  return (
    <div className="stat-cards">
      <StatCard label="RPS" value={r.rps != null ? r.rps.toFixed(1) : "—"} />
      <StatCard label="Avg latency" value={r.avg_ms != null ? r.avg_ms.toFixed(1) : "—"} unit="ms" />
      <StatCard label="p95 latency" value={r.p95_ms != null ? r.p95_ms.toFixed(1) : "—"} unit="ms" />
      <StatCard label="p99 latency" value={r.p99_ms != null ? r.p99_ms.toFixed(1) : "—"} unit="ms" />
      <StatCard
        label="Error rate"
        value={r.error_rate != null ? (r.error_rate * 100).toFixed(2) : "—"}
        unit="%"
        tone={errorTone(r.error_rate)}
      />
      <StatCard label="Total requests" value={r.total_requests != null ? r.total_requests.toLocaleString() : "—"} />
      <StatCard label="CPU peak" value={res.cpu_peak_pct != null ? res.cpu_peak_pct.toFixed(1) : "—"} unit="%" />
      <StatCard label="RAM peak" value={res.mem_peak_mb != null ? res.mem_peak_mb.toFixed(0) : "—"} unit="MB" />
    </div>
  );
}
