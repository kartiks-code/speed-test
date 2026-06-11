import React from "react";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell,
} from "recharts";

const PG_FIELDS = [
  { key: "pg_xact_commit",   label: "xact commit",   color: "#6366f1" },
  { key: "pg_xact_rollback", label: "xact rollback", color: "#ef4444" },
  { key: "pg_blks_read",     label: "blks read",     color: "#f59e0b" },
  { key: "pg_blks_hit",      label: "blks hit",      color: "#22c55e" },
  { key: "pg_tup_inserted",  label: "tup inserted",  color: "#818cf8" },
  { key: "pg_tup_updated",   label: "tup updated",   color: "#34d399" },
  { key: "pg_tup_deleted",   label: "tup deleted",   color: "#fb923c" },
  { key: "pg_tup_fetched",   label: "tup fetched",   color: "#a78bfa" },
];

function CustomTooltip({ active, payload, label }) {
  if (!active || !payload?.length) return null;
  return (
    <div className="custom-tooltip">
      <div className="custom-tooltip-label">{label}</div>
      {payload.map((p) => (
        <div key={p.dataKey} className="custom-tooltip-item">
          <span>{p.name}</span>
          <span style={{ color: p.fill }}>{p.value?.toLocaleString() ?? "—"}</span>
        </div>
      ))}
    </div>
  );
}

export default function PgCountersChart({ pg }) {
  if (!pg) return null;

  const data = PG_FIELDS.map(({ key, label, color }) => ({
    label,
    value: pg[key],
    color,
  })).filter((d) => d.value != null);

  if (!data.length) return null;

  return (
    <div className="chart-card full-width">
      <div className="chart-title">PostgreSQL Counters (delta over run)</div>
      <ResponsiveContainer width="100%" height={240}>
        <BarChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 16 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" vertical={false} />
          <XAxis dataKey="label" tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} width={64} tickFormatter={(v) => v >= 1000 ? `${(v / 1000).toFixed(0)}k` : v} />
          <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(255,255,255,0.04)" }} />
          <Bar dataKey="value" name="count" radius={[4, 4, 0, 0]}>
            {data.map((d, i) => <Cell key={i} fill={d.color} />)}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
