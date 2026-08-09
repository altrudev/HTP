# HTP Governance

HTP is currently an experimental Altru.dev-led open-source protocol project.

The purpose of this governance model is to keep the project open to external implementation and criticism while preserving a coherent protocol during its pre-1.0 development phase.

## Project stewardship

Altru.dev is the initial project steward and maintains the canonical HTP repository and release history.

Project stewardship includes responsibility for:

- accepting or rejecting protocol changes;
- maintaining the normative specification and schemas;
- protecting backward-compatibility expectations;
- coordinating security fixes;
- publishing reference test vectors;
- defining conformance gates;
- managing official HTP releases.

Stewardship does not make one implementation or AI provider authoritative over the protocol's observable semantics.

## Decision model

Before HTP 1.0, protocol decisions use a maintainer-led review model.

Material changes should be supported by:

1. a clearly stated problem;
2. an observable protocol requirement;
3. compatibility analysis;
4. security/privacy analysis where applicable;
5. implementable wire/schema changes;
6. conformance or adversarial tests;
7. documentation of rejected alternatives when the tradeoff is significant.

External proposals are welcome through GitHub issues and pull requests.

## Normative hierarchy

When repository materials disagree, the intended authority order is:

1. the versioned normative HTP specification;
2. the versioned machine-readable schemas;
3. normative conformance test vectors;
4. the reference implementation;
5. explanatory documentation and examples.

A discrepancy between these layers is a project defect and should be resolved explicitly.

## Versioning

HTP uses a protocol version separate from application version numbers.

### Pre-1.0

Versions below 1.0 are experimental. Breaking changes are allowed but must:

- change the affected protocol version;
- include migration/compatibility notes;
- update schemas and test vectors;
- avoid silently changing the meaning of an existing identifier.

### HTP 1.0 gate

HTP should not be declared 1.0 merely because the reference implementation is feature-complete.

A 1.0 release should require, at minimum:

- a frozen canonical wire representation;
- stable signing/canonicalization rules;
- public/private publication semantics;
- replay and lineage rules;
- a documented security model;
- conformance test vectors;
- successful adversarial validation;
- at least one independent or separately implemented interoperability validation;
- a clear extension/version-negotiation model.

## Compatibility

Implementations must not claim conformance to an HTP version while knowingly changing its normative semantics.

Vendor-specific metadata and extensions should be namespaced or otherwise isolated so they cannot be mistaken for portable core HTP fields.

## Releases

Official releases should include:

- a Git tag;
- release notes;
- protocol/schema version information;
- compatibility notes;
- known limitations;
- security-relevant changes;
- reproducible or independently checkable conformance materials where practical.

## Security changes

Security fixes may be developed privately until coordinated disclosure is appropriate. The eventual public fix should document the affected trust boundary without publishing unnecessary exploitation detail.

See [SECURITY.md](SECURITY.md).

## Future governance

If HTP gains multiple independent implementations or substantial external adoption, governance should evolve beyond a single-steward model. Possible future mechanisms include a technical steering group, formal proposal process, compatibility working group, or standards-track submission.

No such external standards status is currently claimed.