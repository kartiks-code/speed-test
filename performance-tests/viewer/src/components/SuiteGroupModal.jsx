import React, { useMemo, useState } from "react";
import {
  stackVariantKey,
  stackVariantLabel,
  pageCountFromAssignments,
  SUITE_PAGE_SIZE,
} from "../lib/suiteGroups.js";

export default function SuiteGroupModal({
  suiteName,
  runs,
  assignments,
  onSave,
  onClose,
}) {
  const [draft, setDraft] = useState(() => ({ ...assignments }));

  const maxSelectablePage = useMemo(() => {
    const fromDraft = pageCountFromAssignments(draft);
    const minPages = Math.ceil(runs.length / SUITE_PAGE_SIZE);
    return Math.max(fromDraft, minPages, 1) + 1;
  }, [draft, runs.length]);

  const pageOptions = useMemo(
    () => Array.from({ length: maxSelectablePage }, (_, i) => i + 1),
    [maxSelectablePage]
  );

  const pageCounts = useMemo(() => {
    const counts = {};
    for (const run of runs) {
      const page = draft[stackVariantKey(run)] ?? 1;
      counts[page] = (counts[page] ?? 0) + 1;
    }
    return counts;
  }, [draft, runs]);

  const overLimitPages = useMemo(
    () => Object.entries(pageCounts).filter(([, count]) => count > SUITE_PAGE_SIZE),
    [pageCounts]
  );

  function handlePageChange(key, page) {
    setDraft((prev) => ({ ...prev, [key]: page }));
  }

  function handleSave() {
    if (overLimitPages.length) return;
    onSave(draft);
  }

  return (
    <div className="modal-overlay" onClick={onClose} role="presentation">
      <div
        className="modal-panel suite-group-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="suite-group-modal-title"
      >
        <div className="modal-header">
          <h3 id="suite-group-modal-title">Change groups — {suiteName}</h3>
          <button type="button" className="modal-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        <p className="modal-description">
          Assign each stack variant to a page ({SUITE_PAGE_SIZE} per page). Your layout is saved
          for this suite.
        </p>

        <div className="suite-group-list">
          {runs.map((run) => {
            const key = stackVariantKey(run);
            return (
              <div key={key} className="suite-group-row">
                <span className="suite-group-label">{stackVariantLabel(run)}</span>
                <select
                  className="control-select suite-group-page-select"
                  value={draft[key] ?? 1}
                  onChange={(e) => handlePageChange(key, Number(e.target.value))}
                >
                  {pageOptions.map((p) => (
                    <option key={p} value={p}>
                      Page {p}
                    </option>
                  ))}
                </select>
              </div>
            );
          })}
        </div>

        {overLimitPages.length > 0 && (
          <p className="modal-error">
            Each page can have at most {SUITE_PAGE_SIZE} runs. Page
            {overLimitPages.length > 1 ? "s" : ""}{" "}
            {overLimitPages.map(([p, c]) => `${p} (${c})`).join(", ")} exceed the limit.
          </p>
        )}

        <div className="modal-actions">
          <button type="button" className="btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="btn-primary" onClick={handleSave} disabled={overLimitPages.length > 0}>
            Save groups
          </button>
        </div>
      </div>
    </div>
  );
}
