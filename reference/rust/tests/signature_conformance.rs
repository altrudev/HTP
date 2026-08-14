// SPDX-License-Identifier: Apache-2.0

use htp_reference::SignedWitnessEnvelope;
use serde_json::Value;

const VECTOR: &str = include_str!("../../../conformance/signatures/v0.2-ed25519-public-01.json");

#[test]
fn canonical_public_signature_vector_verifies() {
    let vector: Value = serde_json::from_str(VECTOR).expect("conformance vector must be valid JSON");
    assert_eq!(
        vector.get("schema").and_then(Value::as_str),
        Some("ddc-htp/conformance-signature-vector/0.1")
    );
    assert_eq!(
        vector.get("vector_id").and_then(Value::as_str),
        Some("htp-signature-v0.2-ed25519-public-01")
    );
    assert_eq!(vector.get("test_vector_only").and_then(Value::as_bool), Some(true));
    assert!(vector.get("test_secret_key_hex").is_none());

    let expected = vector.get("expected").expect("vector expected bindings");
    let envelope: SignedWitnessEnvelope = serde_json::from_value(
        vector.get("envelope").expect("vector envelope").clone(),
    )
    .expect("canonical envelope must deserialize");

    assert_eq!(
        expected.get("key_id").and_then(Value::as_str),
        Some(envelope.key_id.as_str())
    );
    assert_eq!(
        expected.get("public_key_hex").and_then(Value::as_str),
        Some(envelope.public_key_hex.as_str())
    );
    assert_eq!(
        expected.get("witness_hash").and_then(Value::as_str),
        Some(envelope.witness_hash.as_str())
    );
    assert_eq!(
        expected.get("stream_root").and_then(Value::as_str),
        Some(envelope.stream_root.as_str())
    );
    assert_eq!(
        expected.get("content_hash").and_then(Value::as_str),
        Some(envelope.content_hash.as_str())
    );

    let verification = envelope.verify();
    assert!(verification.valid, "issues: {:?}", verification.issues);
}

#[test]
fn canonical_public_signature_vector_detects_tampering() {
    let vector: Value = serde_json::from_str(VECTOR).expect("conformance vector must be valid JSON");
    let mut envelope: SignedWitnessEnvelope = serde_json::from_value(
        vector.get("envelope").expect("vector envelope").clone(),
    )
    .expect("canonical envelope must deserialize");

    envelope.content["stream_root"] = Value::String("33".repeat(32));
    let verification = envelope.verify();
    assert!(!verification.valid);
    assert!(verification
        .issues
        .iter()
        .any(|issue| issue.contains("stream root does not match envelope")));
}
