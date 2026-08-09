# Security Policy

HTP represents authority, provenance, evidence, state change and cryptographic publication. Security defects can therefore affect the trustworthiness of a Human ↔ AI witness even when the underlying conversation appears normal.

## Supported versions

HTP is currently experimental and pre-1.0. Security fixes are applied to the current development line unless a release note explicitly states that an older release remains supported.

## Reporting a vulnerability

Please **do not disclose an exploitable vulnerability in a public GitHub issue before coordinated review**.

Preferred reporting method:

1. Use GitHub's private vulnerability reporting / repository security advisory mechanism when it is available for this repository.
2. Include the affected HTP version or commit.
3. Describe the trust boundary that is violated.
4. Provide a minimal reproduction or test vector when possible.
5. State whether the issue can expose private data, forge authority/evidence, alter witness lineage, bypass validation, replay events, or defeat signature/publication verification.

If private GitHub vulnerability reporting is not available, contact the project owner through the contact channel published by Altru.dev rather than posting exploit details publicly.

## High-priority security classes

Reports are especially important when they involve:

- authority escalation or self-authorization;
- fabricated or misattributed evidence;
- source or actor impersonation;
- event replay or broken ordering;
- predecessor/witness lineage substitution;
- signature forgery, reuse or verification bypass;
- public/private publication leakage;
- canonicalization ambiguity or hash collision opportunities;
- cross-conversation or cross-turn substitution;
- tampering that still produces a valid-looking public witness;
- hidden remote dependencies or unexpected telemetry;
- unsafe handling of signing keys.

## Security model limits

A valid HTP signature demonstrates integrity of the signed HTP material and possession of the corresponding signing key. It does not by itself establish the real-world identity, honesty or security of the signer.

Likewise, HTP can enforce the identity assigned to a configured ingress source, but a compromised source that is already trusted can still submit false information under its legitimate identity. Implementations should therefore treat adapter trust and key management as explicit deployment responsibilities.

## Disclosure

We aim to acknowledge valid security reports, reproduce them, develop regression tests and coordinate disclosure after a fix is available. HTP is experimental, so timelines may vary with severity and complexity.

Security fixes should include an automated regression test whenever practical.