# Human Translation Protocol (HTP)

**A public, machine-readable and human-readable protocol for representing Human ↔ AI transactions.**

HTP makes an AI interaction inspectable without exposing or pretending to expose hidden chain-of-thought. It represents the observable transaction around a conversation: what the human is trying to accomplish, what the AI understood, what authority exists, what evidence entered the interaction, what actions occurred, what changed, what remains uncertain, and how fractures were repaired.

> Status: **experimental**. HTP 0.2 is a working reference implementation, not yet a stable standard.

## Core model

HTP represents:

- **Need** — the human objective and constraints.
- **Projection** — the AI's observable interpretation.
- **Authority** — permissions or capabilities required for actions.
- **Evidence / Action** — observable information and operations.
- **Witness** — the published conclusion and supporting references.
- **Fracture** — ambiguity, contradiction, missing authority, failed action, or invalid assumption.
- **Repair** — clarification, correction, new evidence, revised authority, or changed conclusion.

A central invariant is:

> **Need is not authority.**

A request establishes an objective. It does not automatically authorize every action that could satisfy it.

## What HTP 0.2 includes

- provider-neutral live JSONL event ingestion;
- trusted `source_id → actor` boundaries;
- replay protection;
- explicit human/policy authority grants and denials;
- origin rules that prevent an assistant from self-authorizing, fabricating tool evidence, or claiming effectful execution;
- automatic predecessor-witness lineage across turns;
- cumulative event-stream hashing;
- public/private publication profiles;
- Ed25519 signed witness envelopes;
- deterministic public dimensional change classification;
- Plain, Informed, Expert, and Machine projections;
- browser Human Translator;
- adversarial and end-to-end CLI tests.

HTP does **not** expose hidden chain-of-thought, prove that a conclusion is objectively true, or make a compromised trusted adapter honest.

## Relationship to DDC / Crystalline

HTP is an **independent open-source project built from work originally developed using Dimensional Diffusion Computing (DDC)** as its underlying computational foundation.

DDC and the Crystalline runtime remain separate from this repository.

```text
DDC / Crystalline
      │
      │ foundational computation and private R&D
      ▼
Human Translation Protocol (HTP)
      │
      ├── Human ↔ AI transaction model
      ├── live provenance / authority layer
      ├── signed public/private witnesses
      └── reference tools and projectors
```

HTP retains a stable eight-dimensional integration vocabulary inherited from that architecture:

`semantic · authority · state · resource · security · physical · frequency · lineage`

The public Rust reference implementation computes a transparent HTP dimensional change summary from observable protocol records. **It does not contain or reproduce the private Crystalline/DDC transaction-closure engine, topology propagation, constraint-island machinery, successor-state implementation, or broader runtime internals.**

A full DDC/Crystalline implementation may perform additional analysis outside the canonical HTP witness. HTP validity and signature verification do not require private Altru.dev infrastructure.

## Historical wire identifiers

HTP was first implemented inside the DDC repository. For compatibility, the existing protocol identifiers remain frozen:

- `ddc-htp/0.1` — semantic transaction record;
- `ddc-htp/0.2` — live witness snapshot;
- `ddc-htp-signature/0.2` — signed witness envelope.

The names are historical wire identifiers. They do **not** mean the public HTP implementation contains the private DDC runtime.

## Repository structure

```text
/spec/                   protocol specifications
/schemas/                JSON wire schemas
/reference/rust/         Rust reference implementation
/apps/human-translator/  zero-dependency browser projector
/.github/workflows/      independent build and security-boundary gates
```

## Quick start

Requirements: Rust 1.78 or newer for shipped library/binary targets. Stable Rust is used for the complete test suite.

```bash
cargo test --workspace --all-targets
cargo build --workspace --release
```

The release build produces:

- `htp-translate` — validate/project canonical transactions;
- `htp-live` — ingest a live JSONL event stream and emit witnesses.

### Translate a transaction

```bash
cargo run -p htp-reference --bin htp-translate -- \
  --level plain transaction.json
```

Compare a successor with its predecessor:

```bash
cargo run -p htp-reference --bin htp-translate -- \
  --level expert \
  --previous previous.json \
  current.json
```

### Live unsigned development witness

```bash
cargo run -p htp-reference --bin htp-live -- \
  --trust-policy trust-policy.json \
  --profile public \
  --unsigned \
  events.jsonl
```

### Signed witness

The reference CLI accepts a 32-byte Ed25519 private seed as 64 hexadecimal characters through an environment variable. The key is deliberately not accepted on the command line.

```bash
export HTP_SIGNING_KEY_HEX='<64 hexadecimal characters>'

cargo run -p htp-reference --bin htp-live -- \
  --trust-policy trust-policy.json \
  --profile public \
  --key-id local-htp-signer \
  events.jsonl
```

For migration compatibility, `DDC_HTP_SIGNING_KEY_HEX` is also accepted.

## Human Translator

`apps/human-translator/` is a local, zero-runtime-dependency browser projector. It can display canonical v0.1 transactions, v0.2 public/private witnesses, signed envelopes, unsigned development output, and JSONL captures.

When browser WebCrypto supports Ed25519, the projector verifies signed envelopes locally. The native verifier remains authoritative when browser support is unavailable.

No remote JavaScript or stylesheet is required.

## Specifications and schemas

- [HTP v0.1](spec/HTP-v0.1.md)
- [HTP v0.2](spec/HTP-v0.2.md)
- [v0.1 transaction schema](schemas/ddc-htp-0.1.schema.json)
- [v0.2 event schema](schemas/ddc-htp-0.2-event.schema.json)
- [v0.2 trust-policy schema](schemas/ddc-htp-0.2-trust-policy.schema.json)
- [v0.2 live-snapshot schema](schemas/ddc-htp-0.2-live-snapshot.schema.json)
- [v0.2 signed-envelope schema](schemas/ddc-htp-0.2-envelope.schema.json)

## Security boundary

A configured trusted source can still lie if that source process is compromised. HTP 0.2 binds source identity logically at the adapter boundary; it does not yet provide per-source cryptographic attestation.

Also outside 0.2 are signer rotation/revocation, real-world identity certification, distributed ordering, and selective-disclosure/ZK publication.

Security reports should follow [SECURITY.md](SECURITY.md). Please do not disclose an exploitable vulnerability publicly before coordinated review.

## Versioning

- **HTP 0.1** — structured Human ↔ AI transaction witness.
- **HTP 0.2** — live ingestion, provenance, publication profiles, event-chain integrity, and signed witnesses.

Breaking changes may occur before 1.0. HTP 1.0 is gated by interoperability, independent implementation, canonical serialization, security review, and stable extension/version-negotiation rules rather than by feature count.

See [GOVERNANCE.md](GOVERNANCE.md) and [ROADMAP.md](ROADMAP.md).

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

Unless explicitly stated otherwise, contributions submitted for inclusion are accepted under Apache License 2.0.

## License

HTP source code, protocol documentation, schemas, and reference implementation are licensed under the **Apache License 2.0**, unless a file explicitly states otherwise. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright © 2026 Altru.dev.

The Apache License does not grant permission to use Altru.dev names, logos, marks, or branding beyond reasonable attribution and identification of origin. See [TRADEMARKS.md](TRADEMARKS.md).

---

**Human Translation Protocol (HTP)**  
Developed by Altru.dev.
