import React from "react";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell,
} from "recharts";

const BARS = [
  { key: "avg_ms",  label: "avg",  color: "#6366f1" },
  { key: "p50_ms",  label: "p50",  color: "#818cf8" },
  { key: "p90_ms",  label: "p90",  color: "#f59e0b" },
  { key: "p95_ms",  label: "p95",  color: "#fb923c" },
  { key: "p99_ms",  label: "p99",  color: "#ef4444" },
  { key: "max_ms",  label: "max",  color: "#b91c1c" },
];

function CustomTooltip({ active, payload, label }) {
  if (!active || !payload?.length) return null;
  return (
    <div className="custom-tooltip">
      <div className="custom-tooltip-label">{label}</div>
      {payload.map((p) => (
        <div key={p.dataKey} className="custom-tooltip-item">
          <span>{p.name}</span>
          <span style={{ color: p.fill }}>{p.value != null ? `${p.value.toFixed(2)} ms` : "—"}</span>
        </div>
      ))}
    </div>
  );
}

export default function LatencyChart({ k6 }) {
  const data = BARS.map(({ key, label, color }) => ({
    label,
    value: k6[key],
    color,
  })).filter((d) => d.value != null);

  return (
    <div className="chart-card">
      <div className="chart-title">Latency Breakdown</div>
      <ResponsiveContainer width="100%" height={240}>
        <BarChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" vertical={false} />
          <XAxis dataKey="label" tick={{ fill: "#8892a4", fontSize: 12 }} axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 12 }} axisLine={false} tickLine={false} unit=" ms" width={64} />
          <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(255,255,255,0.04)" }} />
          <Bar dataKey="value" name="latency" radius={[4, 4, 0, 0]}>
            {data.map((d, i) => <Cell key={i} fill={d.color} />)}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
