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
          <span style={{ color: p.fill }}>{p.value}</span>
        </div>
      ))}
    </div>
  );
}

export default function EndpointChecksChart({ endpoints }) {
  if (!endpoints?.length) return null;
  const data = [...endpoints].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="chart-card full-width">
      <div className="chart-title">Per-Endpoint Checks (passes vs fails)</div>
      <ResponsiveContainer width="100%" height={240}>
        <BarChart data={data} margin={{ top: 4, right: 16, bottom: 0, left: 8 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2e3347" vertical={false} />
          <XAxis
            dataKey="name"
            tick={{ fill: "#8892a4", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
            interval={0}
            angle={-20}
            textAnchor="end"
            height={48}
          />
          <YAxis tick={{ fill: "#8892a4", fontSize: 11 }} axisLine={false} tickLine={false} width={48} />
          <Tooltip content={<CustomTooltip />} cursor={{ fill: "rgba(255,255,255,0.04)" }} />
          <Legend wrapperStyle={{ fontSize: 12, color: "#8892a4" }} />
          <Bar dataKey="passes" name="Passes" fill="#22c55e" radius={[3, 3, 0, 0]} />
          <Bar dataKey="fails"  name="Fails"  fill="#ef4444" radius={[3, 3, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
