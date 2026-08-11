# Agentic Benchmark Fixture EXP-0001

Small deterministic JavaScript fixture used by the private Agentic Coding Lab to exercise change-classification diagnosis and repair workflows.

The fixture intentionally has a narrow API surface:

```js
const { classifyChange } = require('./src/classifier');
const result = classifyChange(previousTransaction, currentTransaction);
```

Run the visible tests with:

```bash
npm test
```

The benchmark harness may apply additional evaluator checks outside this target repository. Do not depend on files outside this directory for normal operation.
