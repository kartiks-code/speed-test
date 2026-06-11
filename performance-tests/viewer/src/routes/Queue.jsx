import React, { useEffect, useState, useRef, useCallback } from "react";
import { Link } from "react-router-dom";
import { getQueue, cancelJob, subscribeEvents } from "../lib/api.js";

const STATUS_COLORS = {
  pending:  "#f59e0b",
  running:  "#6366f1",
  done:     "#22c55e",
  failed:   "#ef4444",
  canceled: "#64748b",
};

const STATUS_LABELS = {
  pending:  "Pending",
  running:  "Running",
  done:     "Done",
  failed:   "Failed",
  canceled: "Canceled",
};

function StatusBadge({ status }) {
  return (
    <span
      className="status-badge"
      style={{ background: STATUS_COLORS[status] ?? "#64748b" }}
    >
      {STATUS_LABELS[status] ?? status}
    </span>
  );
}

function MixBar({ mix }) {
  if (!mix) return null;
  const total = (mix.create ?? 0) + (mix.read ?? 0) + (mix.update ?? 0) + (mix.delete ?? 0);
  if (total === 0) return null;
  const ops = [
    { key: "create", color: "#22c55e" },
    { key: "read",   color: "#f59e0b" },
    { key: "update", color: "#6366f1" },
    { key: "delete", color: "#ef4444" },
  ];
  return (
    <div className="mix-bar mini">
      {ops.map(({ key, color }) => {
        const pct = Math.round(((mix[key] ?? 0) / total) * 100);
        return pct > 0 ? (
          <div
            key={key}
            title={`${key} ${pct}%`}
            style={{ width: `${pct}%`, background: color }}
          />
        ) : null;
      })}
    </div>
  );
}

function JobRow({ job, onCancel, isSelected, onSelect }) {
  const duration =
    job.startedAt && job.finishedAt
      ? `${((new Date(job.finishedAt) - new Date(job.startedAt)) / 1000).toFixed(0)}s`
      : job.startedAt
      ? "running…"
      : null;

  return (
    <div
      className={`job-row${isSelected ? " job-row--selected" : ""}${job.status === "running" ? " job-row--running" : ""}`}
      onClick={() => onSelect(job.id)}
    >
      <div className="job-row-main">
        <StatusBadge status={job.status} />
        <span className="job-stack">{job.stackId}</span>
        <span className="job-variant">{job.variant}</span>
        <span className="job-meta">{job.vus} VU · {job.durationSec}s</span>
        <MixBar mix={job.mix} />
        {duration && <span className="job-duration">{duration}</span>}
        {job.runId && job.status === "done" && (
          <Link
            className="btn-link job-results-link"
            to={`/?runId=${job.runId}`}
            onClick={(e) => e.stopPropagation()}
          >
            View results →
          </Link>
        )}
        {job.status === "pending" && (
          <button
            className="btn-cancel"
            onClick={(e) => { e.stopPropagation(); onCancel(job.id); }}
          >
            Cancel
          </button>
        )}
      </div>
    </div>
  );
}

function LogPanel({ job }) {
  const ref = useRef(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (autoScroll && ref.current) {
      ref.current.scrollTop = ref.current.scrollHeight;
    }
  }, [job?.log?.length, autoScroll]);

  if (!job) {
    return (
      <div className="log-panel log-panel--empty">
        <span>Select a job to see its log</span>
      </div>
    );
  }

  return (
    <div className="log-panel">
      <div className="log-panel-header">
        <span className="log-panel-title">
          Log — {job.stackId} {job.variant}
        </span>
        <label className="log-autoscroll">
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={(e) => setAutoScroll(e.target.checked)}
          />
          auto-scroll
        </label>
      </div>
      <div className="log-content" ref={ref}>
        {(job.log ?? []).map((line, i) => (
          <div key={i} className="log-line">{line}</div>
        ))}
        {job.log?.length === 0 && (
          <div className="log-line log-line--dim">Waiting for output…</div>
        )}
      </div>
    </div>
  );
}

export default function Queue() {
  const [jobs, setJobs] = useState([]);
  const [selectedId, setSelectedId] = useState(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState(null);
  const jobsRef = useRef(jobs);
  jobsRef.current = jobs;

  // Load initial queue
  useEffect(() => {
    getQueue()
      .then(setJobs)
      .catch((e) => setError(e.message));
  }, []);

  // SSE subscription
  useEffect(() => {
    const unsub = subscribeEvents((event) => {
      if (event.type === "queue_update") {
        // Merge logs from current in-memory jobs (SSE queue_update omits log array
        // to keep payloads small; logs arrive via separate "log" events)
        setJobs((prev) => {
          const prevMap = Object.fromEntries(prev.map((j) => [j.id, j]));
          return (event.queue ?? []).map((j) => ({
            ...j,
            log: prevMap[j.id]?.log ?? j.log ?? [],
          }));
        });
        setConnected(true);
      } else if (event.type === "log") {
        setJobs((prev) =>
          prev.map((j) =>
            j.id === event.jobId
              ? { ...j, log: [...(j.log ?? []), event.line] }
              : j
          )
        );
      }
    });

    return unsub;
  }, []);

  const handleCancel = useCallback(async (id) => {
    try {
      await cancelJob(id);
    } catch (e) {
      setError(e.message);
    }
  }, []);

  const handleSelect = useCallback((id) => {
    setSelectedId((prev) => (prev === id ? null : id));
  }, []);

  const selectedJob = jobs.find((j) => j.id === selectedId) ?? null;
  const pending = jobs.filter((j) => j.status === "pending").length;
  const running = jobs.find((j) => j.status === "running");

  return (
    <div className="queue-page">
      <div className="queue-header">
        <h2 className="page-title">Test Queue</h2>
        <div className="queue-status-row">
          <div
            className={`conn-dot ${connected ? "conn-dot--on" : "conn-dot--off"}`}
            title={connected ? "Live (SSE connected)" : "Not connected"}
          />
          <span className="queue-stats">
            {running ? (
              <span style={{ color: "#6366f1" }}>Running: {running.stackId} {running.variant}</span>
            ) : (
              <span style={{ color: "#64748b" }}>Idle</span>
            )}
            {pending > 0 && <span> · {pending} pending</span>}
          </span>
          <Link className="btn-secondary btn-sm" to="/run">+ Add run</Link>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      {jobs.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">🗂</div>
          <div>No jobs in queue.</div>
          <div style={{ fontSize: 13, color: "#8892a4" }}>
            <Link to="/run">Add a run</Link> to get started.
          </div>
        </div>
      ) : (
        <div className="queue-layout">
          <div className="job-list">
            {[...jobs].reverse().map((job) => (
              <JobRow
                key={job.id}
                job={job}
                onCancel={handleCancel}
                isSelected={selectedId === job.id}
                onSelect={handleSelect}
              />
            ))}
          </div>
          <LogPanel job={selectedJob} />
        </div>
      )}
    </div>
  );
}
