# Changelog

All notable changes to the Human Translation Protocol project will be documented here.

HTP is currently experimental and pre-1.0. Protocol compatibility notes take precedence over application-level versioning.

## Unreleased

### Independent HTP repository

- Established `altrudev/HTP` as the independent public Apache-2.0 repository for Human Translation Protocol development.
- Migrated the validated HTP 0.1 semantic transaction model and HTP 0.2 live provenance/signature layer into a standalone Rust reference implementation.
- Preserved the historical `ddc-htp/0.1`, `ddc-htp/0.2`, and `ddc-htp-signature/0.2` wire identifiers for compatibility.
- Added standalone `htp-translate` and `htp-live` CLIs.
- Preserved source/actor trust enforcement, replay protection, authority provenance, predecessor witness linkage, event hash-chain receipts, public/private publication profiles, and Ed25519 signatures.
- Migrated the Human Translator browser projector with no mandatory remote JavaScript or stylesheet runtime dependencies.
- Migrated v0.1/v0.2 machine-readable schemas and adversarial/end-to-end tests.
- Added a Cargo lockfile that remains readable by Cargo 1.78 and pinned the crypto dependency edge required by the declared Rust 1.78 shipped-target contract.
- Added strict CI for formatting, Clippy, complete tests, release builds, browser/schema checks, architecture-boundary checks, and Rust 1.78 shipped targets.

### DDC / Crystalline boundary

- Replaced direct dependence on the private Crystalline/DDC transaction-closure implementation with a transparent public HTP dimensional-change classifier.
- Retained the stable eight-dimensional integration vocabulary: semantic, authority, state, resource, security, physical, frequency, and lineage.
- Explicitly excluded private DDC/Crystalline transaction closure, topology propagation, constraint-island machinery, successor-state implementation, and broader runtime internals from the public HTP reference implementation.
- HTP witness validity and signature verification no longer require private Altru.dev infrastructure.
- A full DDC/Crystalline runtime may provide additional non-normative analysis outside the canonical signed HTP witness.

### Project governance

- Adopted Apache License 2.0 for repository source code, protocol documentation, schemas, and reference implementation unless explicitly stated otherwise.
- Added public contribution, governance, security, branding/trademark, changelog, roadmap, issue-template, and pull-request policies.

## HTP 0.2 — original validated implementation

HTP 0.2 introduced:

- live Human ↔ AI event ingestion;
- explicit source/actor trust policy;
- replay protection;
- automatic predecessor-witness linkage;
- authority provenance;
- cumulative event hash-chain receipts;
- public/private publication profiles;
- Ed25519-signed publication envelopes;
- browser-side witness verification;
- v0.2 machine-readable schemas;
- adversarial authority/evidence/signature tests;
- end-to-end live CLI validation.

The independent repository preserves these behaviors while separating HTP protocol/reference code from private DDC core implementation details.

## HTP 0.1 — original validated implementation

HTP 0.1 introduced the structured Human ↔ AI transaction model, including:

- Need;
- AI Projection;
- Authority;
- Evidence;
- Actions;
- Witness;
- Fracture;
- Repair;
- multiple public/expert/machine projections;
- deterministic witness hashing;
- dimensional comparison between successive transaction states.
