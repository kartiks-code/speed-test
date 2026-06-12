import React, { useEffect, useState, useRef, useCallback } from "react";
import { Link } from "react-router-dom";
import { getQueue, cancelJob, clearCompletedJobs, subscribeEvents } from "../lib/api.js";

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

const COMPLETED_STATUSES = new Set(["done", "failed", "canceled"]);

const POLL_MS = 2000;

/** Merge polled/SSE queue rows; preserve in-memory logs when payload omits them. */
function mergeQueueJobs(prev, next) {
  const prevMap = Object.fromEntries(prev.map((j) => [j.id, j]));
  return next.map((j) => ({
    ...j,
    log: j.log?.length ? j.log : (prevMap[j.id]?.log ?? []),
  }));
}

function partitionJobs(jobs) {
  const running = jobs.find((j) => j.status === "running");
  const pending = jobs.filter((j) => j.status === "pending");
  const activeQueue = running ? [running, ...pending] : pending;

  const completed = jobs
    .filter((j) => COMPLETED_STATUSES.has(j.status))
    .sort(
      (a, b) =>
        new Date(b.finishedAt || 0) - new Date(a.finishedAt || 0)
    );

  return { activeQueue, completed };
}

function JobRow({ job, position, onCancel, isSelected, onSelect }) {
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
        {position != null && (
          <span className="job-position" title="Queue position">{position}</span>
        )}
        <StatusBadge status={job.status} />
        {job.suiteName && (
          <span className="job-suite" title="Suite">{job.suiteName}</span>
        )}
        <span className="job-stack">{job.stackId}</span>
        <span className="job-variant">{job.variant}</span>
        <span className="job-meta">{job.vus} VU · {job.durationSec}s</span>
        <MixBar mix={job.mix} />
        {duration && <span className="job-duration">{duration}</span>}
        {job.runId && job.status === "done" && (
          <>
            <Link
              className="btn-link job-results-link"
              to={`/?runId=${job.runId}`}
              onClick={(e) => e.stopPropagation()}
            >
              View →
            </Link>
            {job.suiteName && (
              <Link
                className="btn-link job-results-link"
                to={`/compare?suite=${encodeURIComponent(job.suiteName)}`}
                onClick={(e) => e.stopPropagation()}
              >
                Compare suite →
              </Link>
            )}
          </>
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
        <span>No active job — log will appear automatically when a run starts</span>
      </div>
    );
  }

  return (
    <div className="log-panel">
      <div className="log-panel-header">
        <span className="log-panel-title">
          Log — {job.stackId} {job.variant}
          {job.status === "running" && (
            <span className="log-auto-badge" title="Auto-tracking the running job">live</span>
          )}
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
  // pinnedId: explicitly clicked by user; null = auto-track the running job
  const [pinnedId, setPinnedId] = useState(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState(null);
  const jobsRef = useRef(jobs);
  jobsRef.current = jobs;

  // Load initial queue + poll while this page is open (SSE can miss updates)
  useEffect(() => {
    let cancelled = false;

    const refresh = () =>
      getQueue()
        .then((next) => {
          if (!cancelled) setJobs((prev) => mergeQueueJobs(prev, next));
        })
        .catch((e) => {
          if (!cancelled) setError(e.message);
        });

    refresh();
    const timer = setInterval(refresh, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  // SSE subscription for instant updates and live log lines
  useEffect(() => {
    const unsub = subscribeEvents(
      (event) => {
        if (event.type === "queue_update") {
          setJobs((prev) => mergeQueueJobs(prev, event.queue ?? []));
        } else if (event.type === "log") {
          setJobs((prev) =>
            prev.map((j) =>
              j.id === event.jobId
                ? { ...j, log: [...(j.log ?? []), event.line] }
                : j
            )
          );
        }
      },
      {
        onOpen: () => setConnected(true),
        onClose: () => setConnected(false),
      }
    );

    return unsub;
  }, []);

  // Auto-unpin when the pinned job finishes so the next running job takes over
  useEffect(() => {
    if (!pinnedId) return;
    const pinnedJob = jobsRef.current.find((j) => j.id === pinnedId);
    if (!pinnedJob || COMPLETED_STATUSES.has(pinnedJob.status)) {
      setPinnedId(null);
    }
  }, [jobs, pinnedId]);

  const handleCancel = useCallback(async (id) => {
    try {
      await cancelJob(id);
    } catch (e) {
      setError(e.message);
    }
  }, []);

  const handleClearCompleted = useCallback(async () => {
    try {
      await clearCompletedJobs();
      setPinnedId((prev) => {
        const job = jobsRef.current.find((j) => j.id === prev);
        return job && COMPLETED_STATUSES.has(job.status) ? null : prev;
      });
    } catch (e) {
      setError(e.message);
    }
  }, []);

  // Toggle pin: clicking the already-pinned job unpins (back to auto-track)
  const handleSelect = useCallback((id) => {
    setPinnedId((prev) => (prev === id ? null : id));
  }, []);

  const { activeQueue, completed } = partitionJobs(jobs);
  const pending = activeQueue.filter((j) => j.status === "pending").length;
  const running = activeQueue.find((j) => j.status === "running");

  // Derive selected job: pinned takes priority, otherwise auto-follow running
  const selectedId = pinnedId ?? running?.id ?? null;
  const selectedJob = jobs.find((j) => j.id === selectedId) ?? null;

  return (
    <div className="queue-page">
      <div className="queue-header">
        <h2 className="page-title">Test Queue</h2>
        <div className="queue-status-row">
          <div
            className={`conn-dot ${connected ? "conn-dot--on" : "conn-dot--off"}`}
            title={connected ? "Live (SSE connected)" : "Polling (SSE reconnecting)"}
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
          <div className="job-sections">
            <section className="job-section">
              <h3 className="job-section-title">Queue</h3>
              {activeQueue.length === 0 ? (
                <div className="job-section-empty">No active jobs — add a run or check completed below.</div>
              ) : (
                <div className="job-list">
                  {activeQueue.map((job, i) => (
                    <JobRow
                      key={job.id}
                      job={job}
                      position={i + 1}
                      onCancel={handleCancel}
                      isSelected={selectedId === job.id}
                      onSelect={handleSelect}
                    />
                  ))}
                </div>
              )}
            </section>

            {completed.length > 0 && (
              <section className="job-section">
                <div className="job-section-header">
                  <h3 className="job-section-title">Completed runs</h3>
                  <button
                    type="button"
                    className="btn-secondary btn-sm"
                    onClick={handleClearCompleted}
                  >
                    Clear completed
                  </button>
                </div>
                <div className="job-list">
                  {completed.map((job) => (
                    <JobRow
                      key={job.id}
                      job={job}
                      onCancel={handleCancel}
                      isSelected={selectedId === job.id}
                      onSelect={handleSelect}
                    />
                  ))}
                </div>
              </section>
            )}
          </div>
          <LogPanel job={selectedJob} />
        </div>
      )}
    </div>
  );
}
