'use strict';

class SettingsService {
  constructor({ durable, snapshot }) {
    this.durable = durable;
    this.snapshot = snapshot;
  }

  get(key) {
    return this.snapshot.readThrough(key, this.durable);
  }

  set(key, value) {
    this.durable.write(key, value);
    return value;
  }

  shutdown() {
    this.snapshot.commitTo(this.durable);
  }
}

module.exports = { SettingsService };
