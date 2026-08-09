# Human Translation Protocol v0.2

**Status:** experimental public protocol and reference implementation

HTP v0.2 adds live trust, provenance, publication and signature semantics around the frozen `ddc-htp/0.1` transaction record. The `ddc-htp/*` identifiers are retained as historical wire identifiers so existing v0.1/v0.2 records remain compatible after HTP's extraction into its own repository.

HTP does not expose or claim to expose hidden chain-of-thought. It records the observable transaction around Human ↔ AI interaction: intent, interpretation, authority, evidence, actions, uncertainty, fractures, repairs and published conclusions.

## Architectural boundary

```text
Human / Assistant / Tool / System / Policy adapters
                    ↓
          source_id → actor trust policy
                    ↓
             HTP v0.2 live events
                    ↓
     origin + replay + reference validation
                    ↓
          cumulative event hash chain
                    ↓
       LiveConversationAssembler
                    ↓
        ddc-htp/0.1 semantic record
                    ↓
      validation + portable dimensional change
                    ↓
          LiveWitnessSnapshot v0.2
                    ↓
        public | private publication
                    ↓
       Ed25519 SignedWitnessEnvelope
                    ↓
       public projector / audit consumer
```

### DDC / Crystalline separation

HTP originated as an application/protocol layer inside the private DDC codebase. The public project deliberately keeps only the stable eight-dimensional vocabulary it requires:

- semantic
- authority
- state
- resource
- security
- physical
- frequency
- lineage

The public reference implementation derives a transparent, deterministic HTP dimensional change summary from observable HTP records. It **does not contain or reproduce the private Crystalline/DDC transaction-closure engine, topology propagation, constraint-island machinery or successor-state implementation**.

A full DDC/Crystalline runtime may perform additional non-normative analysis outside the canonical signed HTP witness. This keeps the public protocol independently implementable and keeps DDC as a separate underlying computing project rather than making HTP a container for private DDC internals.

## Core rule

> **Need is not authority.**

A human objective does not automatically grant every capability that could satisfy it. HTP represents objective and authority separately.

## Live event model

The `htp-live` executable consumes JSON Lines. A turn begins with `human_message`; `ai_response` closes it and emits a witness immediately. Successor turns automatically receive the predecessor witness hash.

Supported payload types:

- `human_message`
- `ai_projection`
- `authority_required`
- `authority_grant`
- `authority_deny`
- `authority_revoke`
- `evidence`
- `action`
- `fracture`
- `repair`
- `ai_response`

Accepted event IDs are replay protected.

## Trust policy

Each source ID is bound to one actor class before events are accepted:

```json
{
  "sources": {
    "human-ui": "human",
    "assistant-runtime": "assistant",
    "browser-tool": "tool",
    "local-system": "system",
    "policy-engine": "policy"
  }
}
```

An event is rejected when its claimed actor does not match the actor registered for its source ID.

### Actor-origin rules

| Record | Permitted actor origin |
|---|---|
| Human message | Human |
| AI projection / AI response | Assistant |
| Authority required | Assistant, System, Policy |
| Authority grant / deny / revoke | Human, Policy |
| User-statement evidence | Human |
| Tool-result evidence | Tool |
| System-observation evidence | System |
| External-source evidence | Tool, System |
| Calculation evidence | Assistant, Tool, System |
| Proposed / skipped action | Assistant, Tool, System |
| Authorized action | Human, Policy, System |
| Executed / failed action | Tool, System |
| Fracture | Assistant, Tool, System, Policy |
| Repair | any trusted actor |

These rules are intentionally asymmetric. The assistant may request authority; it may not grant authority to itself. A tool result must originate through a trusted tool source rather than an assistant assertion.

## Authority lifetime

Authority grants and denials can be scoped to:

- `turn` — current turn only;
- `conversation` — inherited by successor turns until changed or revoked.

The semantic validator rejects authorized/executed actions whose required grants are absent.

## Evidence provenance

Origin is checked at ingestion time. For example, a record claiming `kind: tool_result` from an assistant source is rejected before it can enter the transaction.

HTP records observable claims and results, not hidden model reasoning.

## Event hash chain

Each accepted event receives an `EventReceipt` containing event ID, source ID, actor, event type, payload hash and cumulative stream root. The stream root commits to accepted event order and metadata. Reordering, replacing or removing accepted events changes the root.

## Live witness snapshot

```text
LiveWitnessSnapshot
├── protocol_version = ddc-htp/0.2
├── transaction       = canonical ddc-htp/0.1 transaction
├── event_receipts
├── stream_root
└── change            = deterministic HTP change summary
```

The change summary includes changed/conserved dimensions using HTP's public dimensional rules. These fields are portable and reproducible without access to private DDC infrastructure.

## Publication profiles

### Private

The private profile contains the complete snapshot, including raw human statement, constraints, assumptions, source IDs, event IDs, evidence source strings, effect boundaries and resource deltas.

### Public

The public profile removes or replaces:

- raw human statement;
- constraint text;
- projection assumptions;
- evidence source strings → source digests;
- event source IDs;
- raw event IDs → event-ID digests;
- action effect boundaries;
- resource deltas.

It preserves the public semantic witness: objective, AI interpretation, authority state, evidence claims/status, action descriptions/status, conclusion, fractures/repairs, dimensional change summary and event-chain commitments.

The fixed public profile is not a universal DLP system. Human-readable fields deliberately published by the profile must already be safe to disclose.

## Signed witness envelope

HTP v0.2 uses Ed25519:

```text
signature_version = ddc-htp-signature/0.2
algorithm         = ed25519
profile           = public | private
key_id
public_key_hex
witness_hash
stream_root
content_hash
signed_payload_hex
signature_hex
content
```

Verification checks signature metadata, content hash, witness hash, stream root and profile/content binding. A valid envelope cannot be reused as verification for a different transaction.

A valid signature proves integrity and possession of the corresponding private key. It does not by itself establish real-world identity or honesty of the signing runtime.

## Signing-key handling

The reference CLI reads a 32-byte Ed25519 private seed from `HTP_SIGNING_KEY_HEX` as 64 hexadecimal characters. The legacy `DDC_HTP_SIGNING_KEY_HEX` variable is accepted for migration compatibility. Private keys are never accepted as command-line arguments.

## Reference CLI

Build all reference components:

```bash
cargo build --workspace --release
```

Unsigned public stream:

```bash
target/release/htp-live \
  --trust-policy trust-policy.json \
  --profile public \
  --unsigned \
  events.jsonl
```

Signed public stream:

```bash
export HTP_SIGNING_KEY_HEX='<64 hexadecimal characters>'

target/release/htp-live \
  --trust-policy trust-policy.json \
  --profile public \
  --key-id local-htp-signer \
  events.jsonl
```

## Schemas

The `schemas/` directory contains the frozen v0.1 transaction schema and v0.2 trust-policy, event, snapshot and signed-envelope schemas.

## Security limits

HTP v0.2 improves integrity of the recorded interaction; it cannot make a compromised trusted adapter honest. Source IDs are authenticated logically at the configured adapter boundary, not cryptographically by each event.

Still outside v0.2:

- signer key rotation/revocation protocol;
- real-world identity certification for keys;
- per-source cryptographic attestation;
- distributed event ordering;
- selective-disclosure / zero-knowledge publication;
- provider-specific adapters;
- formal external cryptographic review.

HTP v0.2 therefore remains experimental, but it is a functioning live ingestion, provenance and signed-publication protocol layer.
