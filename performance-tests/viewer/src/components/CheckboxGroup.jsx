import React from "react";

/**
 * Multi-select checkbox grid with optional select-all / clear.
 */
export default function CheckboxGroup({
  label,
  options,
  selected,
  onChange,
  disabled = false,
}) {
  const allIds = options.map((o) => o.id);
  const allSelected = allIds.length > 0 && allIds.every((id) => selected.includes(id));
  const someSelected = selected.length > 0 && !allSelected;

  function toggle(id) {
    if (selected.includes(id)) {
      onChange(selected.filter((x) => x !== id));
    } else {
      onChange([...selected, id]);
    }
  }

  function selectAll() {
    onChange([...allIds]);
  }

  function clearAll() {
    onChange([]);
  }

  return (
    <div className="checkbox-group">
      <div className="checkbox-group-header">
        <span className="control-label">{label}</span>
        <span className="checkbox-group-actions">
          <button type="button" className="btn-link" onClick={selectAll} disabled={disabled || allSelected}>
            all
          </button>
          <span className="checkbox-group-sep">·</span>
          <button type="button" className="btn-link" onClick={clearAll} disabled={disabled || !someSelected && !allSelected}>
            none
          </button>
          {selected.length > 0 && (
            <span className="checkbox-group-count">{selected.length} selected</span>
          )}
        </span>
      </div>
      <div className="checkbox-grid">
        {options.map((opt) => (
          <label
            key={opt.id}
            className={`checkbox-item${selected.includes(opt.id) ? " checkbox-item--checked" : ""}${disabled ? " checkbox-item--disabled" : ""}`}
          >
            <input
              type="checkbox"
              checked={selected.includes(opt.id)}
              onChange={() => toggle(opt.id)}
              disabled={disabled}
            />
            <span className="checkbox-item-label">{opt.label}</span>
            {opt.hint && <span className="checkbox-item-hint">{opt.hint}</span>}
          </label>
        ))}
      </div>
    </div>
  );
}
