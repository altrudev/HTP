'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { DurableStore, SessionSnapshot } = require('../src/storage');
const { SettingsService } = require('../src/settings-service');

test('reads a value already present in durable storage', () => {
  const durable = new DurableStore({ theme: 'light' });
  const service = new SettingsService({ durable, snapshot: new SessionSnapshot() });
  assert.equal(service.get('theme'), 'light');
});

test('a newly written key survives a normal restart', () => {
  const durable = new DurableStore();
  const service = new SettingsService({ durable, snapshot: new SessionSnapshot() });
  service.set('locale', 'fr-CA');
  service.shutdown();

  const restarted = new SettingsService({ durable, snapshot: new SessionSnapshot() });
  assert.equal(restarted.get('locale'), 'fr-CA');
});

test('setting one key preserves unrelated durable values', () => {
  const durable = new DurableStore({ theme: 'light', locale: 'en-CA' });
  const service = new SettingsService({ durable, snapshot: new SessionSnapshot() });
  service.set('locale', 'fr-CA');
  service.shutdown();
  assert.equal(durable.read('theme'), 'light');
  assert.equal(durable.read('locale'), 'fr-CA');
});
