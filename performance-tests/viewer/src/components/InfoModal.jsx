import React, { useEffect } from "react";

export function InfoButton({ onClick, label }) {
  return (
    <button
      type="button"
      className="info-icon-btn"
      onClick={onClick}
      aria-label={`Info: ${label}`}
      title={`Learn about ${label} metrics`}
    >
      <svg
        className="info-icon-svg"
        viewBox="0 0 20 20"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
      >
        <circle cx="10" cy="10" r="9" stroke="currentColor" strokeWidth="1.5" />
        <path
          d="M10 9v5"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
        <circle cx="10" cy="6.5" r="0.75" fill="currentColor" />
      </svg>
    </button>
  );
}

export default function InfoModal({ sectionInfo, onClose }) {
  useEffect(() => {
    function onKey(e) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  if (!sectionInfo) return null;

  return (
    <div
      className="modal-overlay"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="modal-panel info-modal-panel"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="info-modal-title"
      >
        <div className="modal-header">
          <h3 id="info-modal-title">{sectionInfo.title}</h3>
          <button
            type="button"
            className="modal-close"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        {sectionInfo.intro && (
          <p className="modal-description">{sectionInfo.intro}</p>
        )}

        <div className="info-modal-body">
          {sectionInfo.columns.map((col) => (
            <div key={col.label} className="info-modal-entry">
              <div className="info-modal-entry-label">{col.label}</div>
              <p className="info-modal-entry-text">{col.explanation}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
