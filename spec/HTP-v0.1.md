# Human Translation Protocol v0.1

**Status:** experimental protocol

Protocol identifier: `ddc-htp/0.1`

The `ddc-htp/0.1` identifier is retained as the frozen historical wire identifier for compatibility. HTP is now an independent project.

HTP v0.1 represents a Human ↔ AI interaction as a public, machine-readable transaction that can be projected at different resolutions without exposing or pretending to expose hidden chain-of-thought.

## Observable transaction

HTP records:

`Need → Projection → Authority → Evidence / Action → Witness → Fracture → Repair`

It does not contain a chain-of-thought field and does not attempt to reconstruct private model reasoning, hidden activations or internal scratchpads.

## Need

The human statement, practical objective and explicit constraints.

> **Need is not authority.**

A human may request an outcome without granting capabilities required to perform external actions.

## Projection

The AI's public operational interpretation, explicit assumptions and ambiguities. Projection is a public interpretation contract, not hidden reasoning.

## Authority

Authority contains `required`, `granted` and `denied` capability sets. An action recorded as `authorized` or `executed` is invalid when any capability in its `requires` set is missing from `granted`.

## Evidence

Evidence records have a stable ID, observable kind, statement, source and status. Status is one of:

- `established`
- `inferred`
- `unknown`
- `contradicted`

Established means established within the represented evidence boundary, not absolute certainty.

## Actions

Action status is one of:

- `proposed`
- `authorized`
- `executed`
- `failed`
- `skipped`

Actions may declare required authority, dimensional effects, resource deltas and reversibility.

## Witness

The witness contains the public conclusion/recommendation plus explicit evidence/action references. The canonical transaction is deterministically SHA-256 hashed to produce its witness hash.

## Fracture and repair

A fracture records ambiguity, contradiction, missing evidence, failed assumption, authority problem or another break in closure. It has one of the eight public DDC-derived dimensions and a severity of `advisory`, `warning` or `blocking`.

A repair explicitly names the fracture it addresses. Repairs are part of lineage; they do not erase history.

## Public states

HTP derives:

- `open`
- `awaiting_authority`
- `blocked`
- `fractured`
- `closed`

Structural validity and public state are separate. A valid transaction may still be fractured or awaiting authority.

## Projection levels

The same canonical transaction can be rendered as:

- **Plain** — objective, interpretation, conclusion, recommendation, unknowns, authority deficit and unresolved fractures.
- **Informed** — adds assumptions, evidence, actions, authority and repair relationships.
- **Expert** — adds identifiers, predecessor link, witness hash, validation and dimensional change information.
- **Machine** — canonical transaction JSON.

Presentation must not silently mutate canonical state.

## Dimensional boundary

HTP uses eight dimensions inherited from its DDC origin:

| Dimension | HTP meaning |
|---|---|
| Semantic | objective, interpretation and public conclusion |
| Authority | required/granted/denied capabilities |
| State | public transaction state within conversation continuity |
| Resource | resource deltas of executed actions |
| Security | security effects of executed actions |
| Physical | physical effects of executed actions |
| Frequency | observational counts of evidence/actions/fractures/repairs |
| Lineage | predecessor, evidence, repairs and witness references |

The independent public HTP implementation defines transparent portable rules for classifying these dimensions. It does **not** contain the private Crystalline/DDC transaction-closure engine. Full DDC analysis may be performed separately without affecting HTP witness validity.

## Authority invariant

For every action whose status is `authorized` or `executed`:

`action.requires ⊆ transaction.authority.granted`

Violation is a validation error.

## Lineage

A successor may contain `predecessor_witness`. The reference CLI warns when a supplied predecessor transaction does not hash to that value.

## CLI

```bash
cargo run -p htp-reference --bin htp-translate -- --level plain transaction.json
```

Compare two states:

```bash
cargo run -p htp-reference --bin htp-translate -- \
  --level expert \
  --previous previous.json \
  current.json
```

Strict validation:

```bash
cargo run -p htp-reference --bin htp-translate -- --strict transaction.json
```

## Security and privacy posture

HTP can operate local-first, requires no telemetry or remote code execution, keeps authority explicit and makes public publication optional. A private witness remains valid HTP.

HTP v0.1 is an experimental protocol foundation, not a standards claim.
