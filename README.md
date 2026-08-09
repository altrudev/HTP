# Human Translation Protocol (HTP)

**A public, machine-readable and human-readable protocol for representing Human ↔ AI transactions.**

HTP is designed to make an AI interaction inspectable without exposing or pretending to expose hidden chain-of-thought. It represents the observable transaction around a conversation: what the human is trying to accomplish, what the AI understood, what authority was available, what evidence was observed, what changed, what remains uncertain, and how fractures were repaired.

> Status: **experimental**. HTP is under active development and is not yet a stable standard.

## Why HTP exists

Ordinary AI conversations are easy to read but difficult to verify, compare, audit, or exchange between systems. HTP adds a structured public witness around the conversation.

The protocol is intended to represent:

- **Need** — the human objective and constraints.
- **Projection** — the AI's observable interpretation of that objective.
- **Authority** — permissions, rules, sources, or capabilities that authorize an action or conclusion.
- **Witness** — observable evidence, actions, state and published conclusions.
- **Fracture** — ambiguity, contradiction, missing authority, failed action, or invalid assumption.
- **Repair** — clarification, correction, new evidence, revised authority or changed conclusion.

A central rule is:

> **Need is not authority.**

A request states an objective. It does not automatically authorize every action that could satisfy that objective.

## Relationship to DDC

HTP is an **independent protocol and project built using Dimensional Diffusion Computing (DDC)** as an underlying computational foundation.

DDC and its Crystalline runtime are separate from this repository. HTP uses DDC concepts such as dimensional state change, transaction closure, lineage and witnesses, but HTP does not redefine DDC itself.

The architectural boundary is intentional:

```text
DDC / Crystalline
      │
      │ foundational computation
      ▼
Human Translation Protocol (HTP)
      │
      ├── live Human ↔ AI event ingestion
      ├── public/private witnesses
      ├── provenance and authority checks
      ├── cryptographic publication envelopes
      └── human-readable projectors
```

This repository is intended to become independently usable and implementable. Public releases must not require access to private Altru.dev infrastructure in order to build, validate or verify the published HTP specification and reference implementation.

## Project goals

HTP aims to provide:

1. A provider-neutral Human ↔ AI transaction format.
2. Explicit separation between human intent and AI interpretation.
3. Explicit separation between objective and authority.
4. Evidence states such as established, inferred, unknown and contradicted.
5. Observable provenance without hidden reasoning disclosure.
6. Stateful fracture and repair across conversation turns.
7. Human-readable, expert and machine-readable projections of the same canonical transaction.
8. Deterministic validation and interoperable test vectors.
9. Verifiable public/private publication profiles.
10. A foundation that can be implemented by multiple AI providers, local systems and independent developers.

## Non-goals

HTP does **not** claim to:

- expose hidden chain-of-thought;
- prove that an AI's conclusion is true;
- prove real-world identity merely from possession of a signing key;
- make a compromised trusted adapter honest;
- replace model-provider safety systems;
- require publication of private conversation data.

## Repository structure

The repository is being initialized for the independent HTP codebase. The intended structure is:

```text
/spec/        protocol specification and normative documents
/schemas/     machine-readable wire schemas
/reference/   reference implementation
/apps/        public Human Translator / projectors
/tests/       interoperability and adversarial tests
/examples/    portable test vectors and sample transactions
/docs/        architecture, security and implementation notes
```

The current validated implementation is being separated from the DDC development repository into this project rather than duplicated as a divergent fork.

## Versioning

Experimental releases use semantic-style protocol versions such as:

- HTP 0.1 — static structured Human ↔ AI transaction witness
- HTP 0.2 — live ingestion, provenance, publication profiles and signed witnesses

Breaking protocol changes may occur before 1.0. A future HTP 1.0 should only be declared after interoperability, security and independent-implementation gates are satisfied.

See [GOVERNANCE.md](GOVERNANCE.md) for the release and compatibility policy.

## Security

HTP deals directly with provenance, authority, evidence and cryptographic verification. Security reports should follow [SECURITY.md](SECURITY.md). Please do not publish an exploitable vulnerability in a public issue before coordinated review.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

Unless explicitly stated otherwise, contributions submitted for inclusion in this repository are accepted under the Apache License 2.0 terms described in the repository license.

## License

HTP source code, protocol documentation and schemas in this repository are licensed under the **Apache License 2.0**, unless a file explicitly states otherwise. See [LICENSE](LICENSE).

Copyright © 2026 Altru.dev.

The Apache License does not grant permission to use Altru.dev names, logos, marks or branding except as necessary to describe the origin of the work. See [TRADEMARKS.md](TRADEMARKS.md).

---

**Human Translation Protocol (HTP)**  
Developed by Altru.dev.