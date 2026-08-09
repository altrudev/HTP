# Changelog

All notable changes to the Human Translation Protocol project will be documented here.

HTP is currently experimental and pre-1.0. Protocol compatibility notes take precedence over application-level versioning.

## Unreleased

### Project separation

- Established `altrudev/HTP` as the independent public repository for Human Translation Protocol development.
- Adopted Apache License 2.0 for repository source code, protocol documentation and schemas unless explicitly stated otherwise.
- Added public contribution, governance, security and branding policies.
- Defined the architectural boundary between HTP and the underlying DDC / Crystalline computing foundation.

### Planned migration

The validated HTP implementation currently developed with the DDC codebase is to be migrated into this repository without changing protocol semantics during the move.

## HTP 0.2 — validated implementation prior to repository separation

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

The independent repository migration will preserve this implementation history while separating HTP application/protocol code from the DDC core repository.

## HTP 0.1 — validated implementation prior to repository separation

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
- DDC dimensional closure comparison between successive transaction states.