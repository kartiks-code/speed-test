/**
 * Grouped comparison bar charts for the Compare page.
 * Each metric shows one bar per selected run, color-coded and with a legend.
 */
import React from "react";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
} from "recharts";

function CustomTooltip({ active, payload, label }) {
  if (!active || !payload?.length) return null;
  return (
    <div className="custom-tooltip">
      <div className="custom-tooltip-label">{label}</div>
      {payload.map((p) => (
        <div key={p.dataKey} className="custom-tooltip-item">
          <span style={{ color: p.fill }}>{p.name}</span>
          <span style={{ color: p.fill }}>
            {p.value != null ? (typeof p.value === "number" ? p.value.toFixed(2) : p.value) : "—"}
            {p.unit ? ` ${p.unit}` : ""}
          </span>
        </div>
      ))}
    </div>
  );
}

// Generic grouped-bar card
function GroupedBarCard({ title, categories, series, unit, yTickFormatter }) {
  // categories: [{ key, label }]  series: [{ label, color, values: {key: number} }]
  const data = categories.map(({ key, label }) => {
    const row = { category: label };
    for (const s of series) row[s.label] = s.values[key];
    return row;
  });

  return (
    <div className="chart-card">
      <div className="chart-title">{title}</div>
      <ResponsiveContainer width="100%" height={260}>
        <BarChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" vertical={false} />
          <XAxis dataKey="category" tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} />
          <YAxis
            tick={{ fill: "#8892a4", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
            width={64}
            unit={unit ? ` ${unit}` : ""}
            tickFormatter={yTickFormatter}
          />
          <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(255,255,255,0.04)" }} />
          <Legend wrapperStyle={{ fontSize: 12, color: "#8892a4" }} />
          {series.map((s) => (
            <Bar key={s.label} dataKey={s.label} fill={s.color} radius={[3, 3, 0, 0]} />
          ))}
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

export function RpsCompareChart({ series }) {
  return (
    <GroupedBarCard
      title="Requests per Second (RPS)"
      categories={[{ key: "rps", label: "RPS" }]}
      series={series.map((s) => ({ ...s, values: { rps: s.run.k6.rps } }))}
      unit="req/s"
    />
  );
}

export function LatencyCompareChart({ series }) {
  const cats = [
    { key: "p50_ms", label: "p50" },
    { key: "p90_ms", label: "p90" },
    { key: "p95_ms", label: "p95" },
    { key: "p99_ms", label: "p99" },
  ];
  return (
    <GroupedBarCard
      title="Latency Percentiles"
      categories={cats}
      series={series.map((s) => ({
        ...s,
        values: {
          p50_ms: s.run.k6.p50_ms,
          p90_ms: s.run.k6.p90_ms,
          p95_ms: s.run.k6.p95_ms,
          p99_ms: s.run.k6.p99_ms,
        },
      }))}
      unit="ms"
    />
  );
}

export function ErrorRateCompareChart({ series }) {
  return (
    <GroupedBarCard
      title="Error Rate"
      categories={[{ key: "error_pct", label: "Error %" }]}
      series={series.map((s) => ({
        ...s,
        values: {
          error_pct: s.run.k6.error_rate != null ? +(s.run.k6.error_rate * 100).toFixed(3) : null,
        },
      }))}
      unit="%"
    />
  );
}

export function CpuCompareChart({ series }) {
  const cats = [
    { key: "cpu_avg_pct", label: "avg" },
    { key: "cpu_peak_pct", label: "peak" },
  ];
  return (
    <GroupedBarCard
      title="CPU Usage"
      categories={cats}
      series={series.map((s) => ({
        ...s,
        values: {
          cpu_avg_pct: s.run.resources.cpu_avg_pct,
          cpu_peak_pct: s.run.resources.cpu_peak_pct,
        },
      }))}
      unit="%"
    />
  );
}

export function RamCompareChart({ series }) {
  const cats = [
    { key: "mem_avg_mb", label: "avg" },
    { key: "mem_peak_mb", label: "peak" },
  ];
  return (
    <GroupedBarCard
      title="RAM Usage"
      categories={cats}
      series={series.map((s) => ({
        ...s,
        values: {
          mem_avg_mb: s.run.resources.mem_avg_mb,
          mem_peak_mb: s.run.resources.mem_peak_mb,
        },
      }))}
      unit="MB"
    />
  );
}

export function PgCompareChart({ series }) {
  const cats = [
    { key: "pg_xact_commit",  label: "xact commit" },
    { key: "pg_blks_read",    label: "blks read" },
    { key: "pg_blks_hit",     label: "blks hit" },
    { key: "pg_tup_inserted", label: "tup inserted" },
    { key: "pg_tup_fetched",  label: "tup fetched" },
  ];
  return (
    <GroupedBarCard
      title="PostgreSQL Counters"
      categories={cats}
      series={series.map((s) => ({
        ...s,
        values: {
          pg_xact_commit:  s.run.pg?.pg_xact_commit,
          pg_blks_read:    s.run.pg?.pg_blks_read,
          pg_blks_hit:     s.run.pg?.pg_blks_hit,
          pg_tup_inserted: s.run.pg?.pg_tup_inserted,
          pg_tup_fetched:  s.run.pg?.pg_tup_fetched,
        },
      }))}
      yTickFormatter={(v) => v >= 1000 ? `${(v / 1000).toFixed(0)}k` : v}
    />
  );
}

export function StartupCompareChart({ series }) {
  const filtered = series.filter((s) => s.run.meta.startup_ms != null);
  if (!filtered.length) return null;
  return (
    <GroupedBarCard
      title="Startup Time"
      categories={[{ key: "startup_ms", label: "startup" }]}
      series={filtered.map((s) => ({
        ...s,
        values: { startup_ms: s.run.meta.startup_ms },
      }))}
      unit="ms"
    />
  );
}
