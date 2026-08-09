# Contributing to HTP

Thank you for contributing to the Human Translation Protocol.

HTP is intended to become independently implementable and interoperable. Contributions should therefore strengthen the protocol rather than make one implementation or AI provider privileged.

## Before opening a change

For substantial protocol changes, open an issue first and describe:

- the problem being solved;
- the observable Human ↔ AI transaction behavior affected;
- whether the change is backward compatible;
- any security or privacy consequences;
- how another independent implementation could reproduce the behavior;
- proposed test vectors or conformance tests.

Bug fixes and documentation corrections may go directly to a pull request when the change is narrow and clear.

## Design principles

Contributions should preserve these principles:

1. **No hidden chain-of-thought dependency.** HTP represents observable transaction state and provenance, not private model reasoning.
2. **Need is not authority.** A user's objective does not automatically authorize an action.
3. **Evidence provenance is explicit.** Claims must not silently change status or source.
4. **Fractures are visible.** Contradiction, ambiguity, missing evidence and missing authority should be represented rather than concealed.
5. **Repair is traceable.** A repair should identify what fracture it resolves.
6. **Provider neutrality.** Core protocol semantics should not require one model vendor, hosted service or proprietary API.
7. **Local verification where practical.** Public witnesses should be inspectable without mandatory telemetry or a remote verification service.
8. **Privacy by design.** Public representations must not accidentally expose private-only fields.
9. **Determinism and interoperability.** Equivalent protocol inputs should produce compatible validation results across implementations.
10. **Security claims must be testable.** New trust or cryptographic claims require adversarial tests.

## Pull requests

A pull request should normally include:

- a focused description of the change;
- affected protocol version(s);
- tests for changed behavior;
- adversarial tests for trust-boundary changes;
- schema/documentation updates when the wire format changes;
- compatibility notes for breaking changes.

Do not mix unrelated refactors with a protocol-semantic change unless necessary.

## Protocol changes

A protocol change is considered breaking when an existing valid transaction, event stream, witness, signature envelope or required interpretation becomes invalid or materially changes meaning.

Before HTP 1.0, breaking changes are permitted, but they must be explicit, documented and versioned.

## Security issues

Do not open a public issue containing a working exploit for an undisclosed vulnerability. Follow [SECURITY.md](SECURITY.md).

## Contribution licensing

Unless you explicitly state otherwise when submitting it, a contribution intentionally submitted for inclusion in HTP is provided under the Apache License 2.0 terms that govern this repository.

Do not submit code, text, data, test vectors or other material that you do not have the right to contribute.

## Attribution

Contributors retain the copyright in their contributions. The project may maintain attribution records in Git history, release notes and NOTICE material where appropriate.

## Conduct

Be technical, specific and respectful. Critique protocol behavior, evidence and implementation choices rather than people. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).