'use strict';

// In-process TTL cache backed by a Map. Entries expire lazily on read so
// there is no background sweep or timer overhead. Each cluster worker has its
// own cache; short TTLs (a few seconds) keep per-worker stale windows small.
class TtlCache {
  constructor() {
    this._store = new Map();
  }

  get(key) {
    const entry = this._store.get(key);
    if (!entry) return undefined;
    if (Date.now() > entry.expiresAt) {
      this._store.delete(key);
      return undefined;
    }
    return entry.value;
  }

  set(key, value, ttlMs) {
    this._store.set(key, { value, expiresAt: Date.now() + ttlMs });
  }

  delete(key) {
    this._store.delete(key);
  }

  deleteByPrefix(prefix) {
    for (const key of this._store.keys()) {
      if (key.startsWith(prefix)) this._store.delete(key);
    }
  }

  clearAll() {
    this._store.clear();
  }
}

module.exports = new TtlCache();
