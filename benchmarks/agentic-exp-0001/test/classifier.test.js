'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { classifyChange } = require('../src/classifier');

function baseTransaction() {
  return {
    conversation_id: 'conv-1',
    state: 'closed',
    need: { statement: 'Help me', objective: 'Make a safe decision', constraints: [] },
    projection: { interpretation: 'Evaluate the request', assumptions: [], ambiguities: [] },
    authority: { required: [], granted: [], denied: [] },
    evidence: [],
    actions: [],
    fractures: [],
    repairs: [],
    lineage: {},
    witness: { conclusion: 'No change required', recommendation: null }
  };
}

test('identical transactions conserve every dimension', () => {
  const previous = baseTransaction();
  const current = structuredClone(previous);
  const result = classifyChange(previous, current);
  assert.deepEqual(result.changed_dimensions, {});
  assert.equal(result.conserved_dimensions.length, 8);
});

test('witness changes are semantic', () => {
  const previous = baseTransaction();
  const current = structuredClone(previous);
  current.witness.conclusion = 'Use the updated recommendation';
  const result = classifyChange(previous, current);
  assert.ok(result.changed_dimensions.semantic);
  assert.ok(!result.conserved_dimensions.includes('semantic'));
});

test('executed resource deltas are classified as resource changes', () => {
  const previous = baseTransaction();
  const current = structuredClone(previous);
  current.actions.push({
    id: 'a-1',
    status: 'executed',
    effects: [],
    resource_delta: { credits: -5 }
  });
  const result = classifyChange(previous, current);
  assert.deepEqual(result.changed_dimensions.resource, ['resource:credits']);
  assert.ok(result.changed_dimensions.frequency);
});
