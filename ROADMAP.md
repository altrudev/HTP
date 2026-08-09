# HTP Roadmap

HTP is now an independent public protocol/reference project. The immediate extraction goal is complete: HTP builds and verifies without access to private DDC/Crystalline runtime internals while preserving its historical wire formats and security model.

## Phase 1 — Repository foundation

Status: **complete**

- Apache-2.0 licensing and NOTICE
- public README and DDC/HTP architectural boundary
- security policy
- contribution policy
- governance/versioning policy
- branding/trademark boundary
- issue and pull-request templates
- changelog and roadmap

## Phase 2 — Controlled implementation migration

Status: **complete**

- HTP 0.1 semantic transaction model migrated;
- HTP 0.2 live provenance/signature model migrated;
- historical wire identifiers preserved;
- schemas and adversarial tests preserved;
- Human Translator projector migrated;
- standalone `htp-translate` and `htp-live` CLIs established;
- SPDX headers added to migrated source;
- private repository assumptions removed from canonical HTP verification.

## Phase 3 — Independent-build boundary

Status: **complete for HTP 0.2 reference implementation**

Current gates:

- public dependencies only;
- independently parseable schemas;
- reference and adversarial tests;
- independently verifiable signed public/private witnesses;
- local browser projector without mandatory remote runtime dependencies;
- strict architecture-boundary CI preventing private Crystalline/DDC closure internals from entering the public reference code;
- reproducible Cargo lockfile;
- stable Rust CI plus Rust 1.78 shipped-target compatibility.

HTP retains the eight-dimensional DDC-derived vocabulary but performs its canonical public change classification through transparent HTP rules. Full Crystalline/DDC analysis remains separate and non-normative to HTP witness validity.

## Phase 4 — Interoperability package

Next priority.

- canonical portable test vectors;
- provider-neutral live event fixtures;
- conformance runner;
- known-good signed witness examples;
- failure vectors for authority escalation, fabricated evidence, replay, lineage substitution and publication leakage;
- canonical serialization rules suitable for independent implementations;
- implementation guidance for non-Rust clients.

## Phase 5 — Runtime and provider adapters

Adapters should translate only observable provider/runtime events into HTP without changing core protocol semantics.

Priorities:

- generic local JSON/JSONL event bridge;
- local LLM adapter;
- hosted model API adapter examples;
- trusted tool-execution adapter;
- browser/local daemon live stream into the Human Translator;
- multi-agent aggregation experiments after single-session semantics stabilize.

Vendor-specific fields must remain isolated from portable HTP semantics. A model must never be allowed to choose its own trusted actor identity.

## Phase 6 — Key and publication lifecycle

- signer key rotation;
- revocation records;
- local key-store guidance;
- backup/restore procedure;
- deterministic publication-policy manifests;
- sensitivity labels and secret scanning before public publication;
- redaction linkage and compatibility rules.

## Phase 7 — HTP 1.0 readiness

HTP 1.0 is gated by interoperability and security rather than feature count.

Minimum objectives:

- stable canonical serialization;
- stable signature envelope;
- stable publication/redaction semantics;
- formal extension/version-negotiation mechanism;
- conformance suite;
- at least one independent implementation outside the Rust reference implementation;
- adversarial security review;
- documented key-rotation/revocation strategy;
- documented optional DDC integration boundary;
- no hidden dependency on Altru.dev-hosted services.

## Research directions

The following are research directions rather than prerequisites for the next release:

- per-source cryptographic attestation;
- selective-disclosure / zero-knowledge publication;
- distributed event ordering;
- multi-party witness consensus;
- formal certification marks;
- standards-body submission.

These should not delay the simpler interoperability foundation.
