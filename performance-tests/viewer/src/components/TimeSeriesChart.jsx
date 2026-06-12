import React from "react";
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
} from "recharts";

function CustomTooltip({ active, payload, label }) {
  if (!active || !payload?.length) return null;
  return (
    <div className="custom-tooltip">
      <div className="custom-tooltip-label">t = {label}s</div>
      {payload.map((p) => (
        <div key={p.dataKey} className="custom-tooltip-item">
          <span style={{ color: p.color }}>{p.name}</span>
          <span style={{ color: p.color }}>
            {p.value != null ? `${p.value.toFixed(1)} ${p.unit || ""}` : "—"}
          </span>
        </div>
      ))}
    </div>
  );
}

export function CpuTimeSeriesChart({ timeseries }) {
  if (!timeseries?.length) return null;
  return (
    <div className="chart-card">
      <div className="chart-title">CPU Usage During k6 Load Test</div>
      <ResponsiveContainer width="100%" height={220}>
        <LineChart data={timeseries} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" />
          <XAxis dataKey="t" tick={{ fill: "#8892a4", fontSize: 11 }} unit="s" axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} unit="%" width={48} />
          <Tooltip content={<CustomTooltip />} />
          <Line type="monotone" dataKey="cpu_pct" name="CPU" stroke="#6366f1" dot={false} strokeWidth={2} unit="%" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

export function RamTimeSeriesChart({ timeseries }) {
  if (!timeseries?.length) return null;
  return (
    <div className="chart-card">
      <div className="chart-title">RAM Usage During k6 Load Test</div>
      <ResponsiveContainer width="100%" height={220}>
        <LineChart data={timeseries} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" />
          <XAxis dataKey="t" tick={{ fill: "#8892a4", fontSize: 11 }} unit="s" axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} unit=" MB" width={64} />
          <Tooltip content={<CustomTooltip />} />
          <Line type="monotone" dataKey="mem_mb" name="RAM" stroke="#22c55e" dot={false} strokeWidth={2} unit=" MB" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

// Multi-series overlay for comparison page
export function CpuOverlayChart({ series }) {
  // series: [{ label, color, data: [{t, cpu_pct}] }]
  if (!series?.length) return null;

  // Merge all series into a single array keyed by t
  const tMap = {};
  for (const s of series) {
    for (const pt of s.data) {
      if (!tMap[pt.t]) tMap[pt.t] = { t: pt.t };
      tMap[pt.t][s.label] = pt.cpu_pct;
    }
  }
  const data = Object.values(tMap).sort((a, b) => a.t - b.t);

  return (
    <div className="chart-card full-width">
      <div className="chart-title">CPU Usage Over Time (overlay)</div>
      <ResponsiveContainer width="100%" height={260}>
        <LineChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" />
          <XAxis dataKey="t" tick={{ fill: "#8892a4", fontSize: 11 }} unit="s" axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} unit="%" width={48} />
          <Tooltip content={<CustomTooltip />} />
          <Legend wrapperStyle={{ fontSize: 12, color: "#8892a4" }} />
          {series.map((s) => (
            <Line key={s.label} type="monotone" dataKey={s.label} stroke={s.color} dot={false} strokeWidth={2} unit="%" />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

export function RamOverlayChart({ series }) {
  if (!series?.length) return null;

  const tMap = {};
  for (const s of series) {
    for (const pt of s.data) {
      if (!tMap[pt.t]) tMap[pt.t] = { t: pt.t };
      tMap[pt.t][s.label] = pt.mem_mb;
    }
  }
  const data = Object.values(tMap).sort((a, b) => a.t - b.t);

  return (
    <div className="chart-card full-width">
      <div className="chart-title">RAM Usage Over Time (overlay)</div>
      <ResponsiveContainer width="100%" height={260}>
        <LineChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" />
          <XAxis dataKey="t" tick={{ fill: "#8892a4", fontSize: 11 }} unit="s" axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} unit=" MB" width={64} />
          <Tooltip content={<CustomTooltip />} />
          <Legend wrapperStyle={{ fontSize: 12, color: "#8892a4" }} />
          {series.map((s) => (
            <Line key={s.label} type="monotone" dataKey={s.label} stroke={s.color} dot={false} strokeWidth={2} unit=" MB" />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
