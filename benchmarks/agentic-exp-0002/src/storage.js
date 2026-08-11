'use strict';

class DurableStore {
  constructor(seed = {}) {
    this.values = new Map(Object.entries(seed));
  }

  read(key) {
    return this.values.has(key) ? this.values.get(key) : undefined;
  }

  write(key, value) {
    this.values.set(key, value);
  }

  snapshot() {
    return Object.fromEntries(this.values.entries());
  }
}

class SessionSnapshot {
  constructor() {
    this.values = new Map();
  }

  readThrough(key, durable) {
    if (!this.values.has(key)) {
      const value = durable.read(key);
      if (value !== undefined) this.values.set(key, value);
    }
    return this.values.get(key);
  }

  commitTo(durable) {
    for (const [key, value] of this.values.entries()) {
      durable.write(key, value);
    }
  }
}

module.exports = { DurableStore, SessionSnapshot };
