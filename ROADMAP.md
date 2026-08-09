# HTP Roadmap

HTP is being separated from the DDC development repository into an independent public protocol project. The immediate goal is to preserve the already-validated HTP behavior while creating a clean public implementation boundary.

## Phase 1 — Repository foundation

Status: **complete**

- Apache-2.0 licensing and NOTICE
- public README and DDC/HTP architectural boundary
- security policy
- contribution policy
- governance/versioning policy
- branding/trademark boundary
- issue and pull-request templates
- changelog

## Phase 2 — Controlled implementation migration

- migrate HTP 0.1 and 0.2 protocol modules without semantic redesign;
- preserve validated schemas and adversarial tests;
- preserve the Human Translator projector;
- retain the `ddc-htp-live` reference CLI behavior;
- add SPDX license headers to migrated source files;
- remove repository-local assumptions that require access to private DDC development infrastructure;
- expose only the stable DDC interface HTP actually needs;
- reproduce all previous HTP validation results in this repository before declaring migration complete.

## Phase 3 — Independent-build boundary

HTP must become buildable and testable by an external developer using only public dependencies.

Required gates:

- fresh clone builds without Altru.dev private infrastructure;
- schemas validate independently;
- reference tests and adversarial tests pass;
- signed public/private witnesses verify independently;
- browser projector remains local and free of mandatory remote runtime dependencies;
- DDC integration is through a documented public interface rather than copied internal implementation details.

## Phase 4 — Interoperability package

- canonical portable test vectors;
- provider-neutral live event fixtures;
- conformance runner;
- known-good signed witness examples;
- failure vectors for authority escalation, fabricated evidence, replay, lineage substitution and publication leakage;
- implementation guidance for non-Rust clients.

## Phase 5 — Provider adapters

Adapters should translate observable provider events into HTP without changing core protocol semantics.

Candidates include:

- generic JSON/JSONL adapter;
- local LLM adapter;
- hosted model API adapter examples;
- agent/tool-execution adapter;
- multi-agent aggregation experiments.

Vendor-specific fields must remain isolated from portable HTP semantics.

## Phase 6 — HTP 1.0 readiness

HTP 1.0 is gated by interoperability and security rather than feature count.

Minimum objectives:

- stable canonical serialization;
- stable signature envelope;
- stable publication/redaction semantics;
- formal extension/version-negotiation mechanism;
- conformance suite;
- independent implementation validation;
- adversarial security review;
- documented key-rotation/revocation strategy;
- stable public DDC dependency/interface;
- no hidden dependency on Altru.dev-hosted services.

## Not on the critical path

The following are research directions rather than prerequisites for the next release:

- selective-disclosure / zero-knowledge publication;
- distributed event ordering;
- multi-party witness consensus;
- formal certification marks;
- standards-body submission.

These should not delay the simpler interoperability foundation.