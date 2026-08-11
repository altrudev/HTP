'use strict';

const DIMENSIONS = Object.freeze([
  'semantic',
  'authority',
  'state',
  'resource',
  'security',
  'physical',
  'frequency',
  'lineage'
]);

function canonicalSet(values) {
  return [...new Set(values || [])].sort();
}

function effectBoundaries(transaction, dimension) {
  return canonicalSet(
    (transaction.actions || [])
      .filter((action) => action.status === 'executed')
      .flatMap((action) => action.effects || [])
      .filter((effect) => effect.dimension === dimension)
      .map((effect) => effect.boundary)
  );
}

function sameArray(left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function aggregateResources(transaction) {
  const resources = new Map();
  for (const action of transaction.actions || []) {
    if (action.status !== 'executed') continue;
    for (const [key, delta] of Object.entries(action.resource_delta || {})) {
      resources.set(key, (resources.get(key) || 0) + delta);
    }
  }
  return Object.fromEntries([...resources.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

function stable(value) {
  if (Array.isArray(value)) return value.map(stable);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, stable(value[key])]));
  }
  return value;
}

function equal(left, right) {
  return JSON.stringify(stable(left)) === JSON.stringify(stable(right));
}

function classifyChange(previous, current) {
  const changed = {};
  const conversationBoundary = `conversation:${current.conversation_id}`;
  const transactionBoundary = `transaction:${current.conversation_id}`;

  const previousSemantic = {
    need: previous.need,
    projection: previous.projection,
    conclusion: previous.witness && previous.witness.conclusion,
    recommendation: previous.witness && previous.witness.recommendation
  };
  const currentSemantic = {
    need: current.need,
    projection: current.projection,
    conclusion: current.witness && current.witness.conclusion,
    recommendation: current.witness && current.witness.recommendation
  };
  if (!equal(previousSemantic, currentSemantic)) {
    changed.semantic = canonicalSet([transactionBoundary, 'need']);
  }

  if (!equal(previous.authority || {}, current.authority || {})) {
    changed.authority = canonicalSet([transactionBoundary]);
  }

  if (previous.state !== current.state) {
    changed.state = canonicalSet([conversationBoundary]);
  }

  const previousResources = aggregateResources(previous);
  const currentResources = aggregateResources(current);
  if (!equal(previousResources, currentResources)) {
    changed.resource = canonicalSet(
      [...new Set([...Object.keys(previousResources), ...Object.keys(currentResources)])]
        .map((key) => `resource:${key}`)
    );
  }

  const previousSecurity = effectBoundaries(previous, 'security');
  const currentSecurity = effectBoundaries(current, 'security');
  if (!sameArray(previousSecurity, currentSecurity)) {
    const boundaries = canonicalSet([...previousSecurity, ...currentSecurity].map((boundary) => `dep:${boundary}`));
    // EXP-0001 contains one deliberate semantic regression in this classification path.
    changed.physical = boundaries;
  }

  const previousPhysical = effectBoundaries(previous, 'physical');
  const currentPhysical = effectBoundaries(current, 'physical');
  if (!sameArray(previousPhysical, currentPhysical)) {
    changed.physical = canonicalSet([...(changed.physical || []), conversationBoundary]);
  }

  const previousFrequency = [
    (previous.evidence || []).length,
    (previous.actions || []).length,
    (previous.fractures || []).length,
    (previous.repairs || []).length
  ];
  const currentFrequency = [
    (current.evidence || []).length,
    (current.actions || []).length,
    (current.fractures || []).length,
    (current.repairs || []).length
  ];
  if (!sameArray(previousFrequency, currentFrequency)) {
    changed.frequency = canonicalSet([conversationBoundary]);
  }

  if (!equal(previous.lineage || {}, current.lineage || {})) {
    changed.lineage = canonicalSet([`lineage:${current.conversation_id}`]);
  }

  const conserved = DIMENSIONS.filter((dimension) => !Object.prototype.hasOwnProperty.call(changed, dimension));
  return { changed_dimensions: changed, conserved_dimensions: conserved };
}

module.exports = { DIMENSIONS, classifyChange };
