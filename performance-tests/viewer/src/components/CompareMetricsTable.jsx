import React, { useMemo, useState } from "react";
import InfoModal, { InfoButton } from "./InfoModal.jsx";
import { SECTION_INFO } from "../lib/sectionInfo.js";

const COL_RUN = { key: "label", label: "Run", type: "string", sticky: true };

const PERF_COLUMNS = [
  { key: "rps",        label: "RPS",       type: "number", unit: "req/s", decimals: 1, higherIsBetter: true,  deltaLabel: "% behind leader" },
  { key: "error_pct",  label: "Error %",   type: "number", unit: "%",     decimals: 3, higherIsBetter: false, deltaLabel: "% more errors vs leader" },
  { key: "p50_ms",     label: "p50",       type: "number", unit: "ms",    decimals: 1, higherIsBetter: false, deltaLabel: "% longer than leader" },
  { key: "p90_ms",     label: "p90",       type: "number", unit: "ms",    decimals: 1, higherIsBetter: false, deltaLabel: "% longer than leader" },
  { key: "p95_ms",     label: "p95",       type: "number", unit: "ms",    decimals: 1, higherIsBetter: false, deltaLabel: "% longer than leader" },
  { key: "p99_ms",     label: "p99",       type: "number", unit: "ms",    decimals: 1, higherIsBetter: false, deltaLabel: "% longer than leader" },
  { key: "startup_ms", label: "Startup",   type: "number", unit: "ms",    decimals: 0, higherIsBetter: false, deltaLabel: "% slower startup vs leader" },
];

const RESOURCE_COLUMNS = [
  { key: "mem_avg_mb",   label: "RAM avg",  type: "number", unit: "MB", decimals: 1, higherIsBetter: false, deltaLabel: "% more RAM vs leader"  },
  { key: "mem_peak_mb",  label: "RAM peak", type: "number", unit: "MB", decimals: 1, higherIsBetter: false, deltaLabel: "% more RAM vs leader"  },
  { key: "cpu_avg_pct",  label: "CPU avg",  type: "number", unit: "%",  decimals: 1, higherIsBetter: false, deltaLabel: "% more CPU vs leader"  },
  { key: "cpu_peak_pct", label: "CPU peak", type: "number", unit: "%",  decimals: 1, higherIsBetter: false, deltaLabel: "% more CPU vs leader"  },
];

const DB_COLUMNS = [
  { key: "pg_xact_commit",  label: "PG xact commit", type: "number" },
  { key: "pg_blks_read",    label: "PG blks read",   type: "number" },
  { key: "pg_blks_hit",     label: "PG blks hit",    type: "number" },
  { key: "pg_tup_inserted", label: "PG tup ins",     type: "number" },
  { key: "pg_tup_fetched",  label: "PG tup fetch",   type: "number" },
];

const SECTIONS = [
  { title: "Performance Statistics", columns: PERF_COLUMNS,     defaultSort: "rps" },
  { title: "Resource Usage Metrics", columns: RESOURCE_COLUMNS, defaultSort: "mem_avg_mb" },
  { title: "DB Metrics",             columns: DB_COLUMNS,        defaultSort: "pg_xact_commit" },
];

function seriesToRow(s) {
  const k6 = s.run.k6 ?? {};
  const res = s.run.resources ?? {};
  const pg  = s.run.pg ?? {};
  return {
    id:             s.run.meta.run_id,
    label:          s.label,
    color:          s.color,
    rps:            k6.rps,
    error_pct:      k6.error_rate != null ? +(k6.error_rate * 100).toFixed(3) : null,
    p50_ms:         k6.p50_ms,
    p90_ms:         k6.p90_ms,
    p95_ms:         k6.p95_ms,
    p99_ms:         k6.p99_ms,
    startup_ms:     s.run.meta.startup_ms ?? null,
    cpu_avg_pct:    res.cpu_avg_pct,
    cpu_peak_pct:   res.cpu_peak_pct,
    mem_avg_mb:     res.mem_avg_mb,
    mem_peak_mb:    res.mem_peak_mb,
    pg_xact_commit:  pg.pg_xact_commit,
    pg_blks_read:    pg.pg_blks_read,
    pg_blks_hit:     pg.pg_blks_hit,
    pg_tup_inserted: pg.pg_tup_inserted,
    pg_tup_fetched:  pg.pg_tup_fetched,
  };
}

function compareValues(a, b, type) {
  const aNull = a == null || a === "";
  const bNull = b == null || b === "";
  if (aNull && bNull) return 0;
  if (aNull) return 1;
  if (bNull) return -1;
  if (type === "string") return String(a).localeCompare(String(b));
  return a - b;
}

function formatCell(value, col) {
  if (value == null) return "—";
  if (col.type === "string") return value;
  if (col.decimals != null) return value.toFixed(col.decimals);
  if (Math.abs(value) >= 1000) return value.toLocaleString(undefined, { maximumFractionDigits: 0 });
  return String(value);
}

function SortIndicator({ active, dir }) {
  if (!active) return <span className="compare-table-sort-idle">↕</span>;
  return <span className="compare-table-sort-active">{dir === "asc" ? "↑" : "↓"}</span>;
}

function DeltaCell({ value, bestValue, col }) {
  if (value == null || bestValue == null) return <span className="delta-na">—</span>;

  // When the leader's value is 0 we can't compute a relative % — fall back to
  // showing the absolute gap above the leader (e.g. "+0.003%" for error rate).
  if (bestValue === 0) {
    if (value === 0) return <span className="delta-leader">0%</span>;
    const abs = col.decimals != null
      ? value.toFixed(col.decimals)
      : (Math.abs(value) >= 1000
          ? value.toLocaleString(undefined, { maximumFractionDigits: 0 })
          : String(value));
    return <span className="delta-high">+{abs}{col.unit ? col.unit : ""}</span>;
  }

  const delta = col.higherIsBetter
    ? (bestValue - value) / bestValue * 100
    : (value - bestValue) / bestValue * 100;

  if (delta < 0.05) return <span className="delta-leader">0%</span>;
  const cls = delta < 20 ? "delta-low" : delta < 60 ? "delta-mid" : "delta-high";
  return <span className={cls}>+{delta.toFixed(1)}%</span>;
}

function buildExtendedCols(columns) {
  const result = [COL_RUN];
  for (const col of columns) {
    result.push(col);
    if (col.type === "number" && col.deltaLabel != null) {
      result.push({ key: `${col.key}__delta`, label: col.deltaLabel, type: "delta", sourceKey: col.key, isDelta: true });
    }
  }
  return result;
}

function SectionTable({ title, columns, rows }) {
  const [sortKey, setSortKey] = useState(columns[0]?.key ?? "label");
  const [sortDir, setSortDir] = useState("desc");
  const [showInfo, setShowInfo] = useState(false);

  const extCols = useMemo(() => buildExtendedCols(columns), [columns]);
  const sortableCols = [COL_RUN, ...columns];

  const bestValues = useMemo(() => {
    const best = {};
    for (const col of columns) {
      if (col.type !== "number") continue;
      const vals = rows.map((r) => r[col.key]).filter((v) => v != null);
      if (!vals.length) continue;
      best[col.key] = col.higherIsBetter ? Math.max(...vals) : Math.min(...vals);
    }
    return best;
  }, [rows, columns]);

  const sortedRows = useMemo(() => {
    const col = sortableCols.find((c) => c.key === sortKey) ?? sortableCols[0];
    return [...rows].sort((a, b) => {
      const cmp = compareValues(a[col.key], b[col.key], col.type);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [rows, sortKey, sortDir]);

  function handleSort(key) {
    if (key === sortKey) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(key === "label" ? "asc" : "desc");
    }
  }

  return (
    <div className="compare-section">
      <div className="compare-section-title">
        <span>{title}</span>
        <InfoButton onClick={() => setShowInfo(true)} label={title} />
      </div>
      {showInfo && (
        <InfoModal
          sectionInfo={SECTION_INFO[title]}
          onClose={() => setShowInfo(false)}
        />
      )}
      <div className="compare-metrics-table-wrap">
        <table className="compare-metrics-table">
          <thead>
            <tr>
              {extCols.map((col) => {
                if (col.isDelta) {
                  return (
                    <th key={col.key} className="compare-table-th delta" title={col.label}>
                      <span className="compare-table-th-inner">{col.label}</span>
                    </th>
                  );
                }
                return (
                  <th
                    key={col.key}
                    className={`compare-table-th${col.sticky ? " sticky" : ""}${sortKey === col.key ? " sorted" : ""}`}
                    onClick={() => handleSort(col.key)}
                    title={`Sort by ${col.label}`}
                  >
                    <span className="compare-table-th-inner">
                      {col.label}
                      {col.unit && <span className="compare-table-unit"> ({col.unit})</span>}
                      <SortIndicator active={sortKey === col.key} dir={sortDir} />
                    </span>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {sortedRows.map((row) => (
              <tr key={row.id}>
                {extCols.map((col) => {
                  if (col.isDelta) {
                    const srcCol = columns.find((c) => c.key === col.sourceKey);
                    return (
                      <td key={col.key} className="compare-table-td delta num">
                        {srcCol ? (
                          <DeltaCell value={row[col.sourceKey]} bestValue={bestValues[col.sourceKey]} col={srcCol} />
                        ) : "—"}
                      </td>
                    );
                  }
                  return (
                    <td
                      key={col.key}
                      className={`compare-table-td${col.sticky ? " sticky" : ""}${col.type === "number" ? " num" : ""}`}
                    >
                      {col.key === "label" ? (
                        <span className="compare-table-run-label">
                          <span className="compare-table-dot" style={{ background: row.color }} />
                          {row.label}
                        </span>
                      ) : (
                        formatCell(row[col.key], col)
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default function CompareMetricsTable({ series }) {
  const rows = useMemo(() => series.map(seriesToRow), [series]);

  return (
    <div>
      {SECTIONS.map((section) => (
        <SectionTable
          key={section.title}
          title={section.title}
          columns={section.columns}
          rows={rows}
        />
      ))}
    </div>
  );
}
