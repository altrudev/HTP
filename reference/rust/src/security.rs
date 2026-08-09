// SPDX-License-Identifier: Apache-2.0

use crate::dimension::Dimension;
use crate::live::{EventReceipt, LiveWitnessSnapshot, HTP_LIVE_VERSION};
use crate::protocol::HumanAiTransaction;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

pub const HTP_SIGNATURE_VERSION: &str = "ddc-htp-signature/0.2";
pub const HTP_SIGNATURE_ALGORITHM: &str = "ed25519";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationProfile {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedWitnessEnvelope {
    pub signature_version: String,
    pub algorithm: String,
    pub profile: PublicationProfile,
    pub key_id: String,
    pub public_key_hex: String,
    pub witness_hash: String,
    pub stream_root: String,
    pub content_hash: String,
    pub signed_payload_hex: String,
    pub signature_hex: String,
    pub content: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub valid: bool,
    #[serde(default)]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureError {
    pub code: String,
    pub message: String,
}

impl SignatureError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SignatureError {}

#[derive(Serialize)]
struct SignaturePayload<'a> {
    signature_version: &'a str,
    algorithm: &'a str,
    profile: PublicationProfile,
    key_id: &'a str,
    public_key_hex: &'a str,
    witness_hash: &'a str,
    stream_root: &'a str,
    content_hash: &'a str,
}

impl SignedWitnessEnvelope {
    pub fn sign(
        snapshot: &LiveWitnessSnapshot,
        profile: PublicationProfile,
        key_id: impl Into<String>,
        secret_key_hex: &str,
    ) -> Result<Self, SignatureError> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() {
            return Err(SignatureError::new(
                "missing_key_id",
                "key_id must not be empty",
            ));
        }
        let secret = parse_fixed_hex::<32>(secret_key_hex)
            .map_err(|message| SignatureError::new("invalid_signing_key", message))?;
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key_hex = encode_hex(&signing_key.verifying_key().to_bytes());
        let content = publish_snapshot(snapshot, profile);
        let content_hash = hash_json(&content);
        let witness_hash = snapshot.witness_hash();
        let stream_root = snapshot.stream_root.clone();

        let payload = SignaturePayload {
            signature_version: HTP_SIGNATURE_VERSION,
            algorithm: HTP_SIGNATURE_ALGORITHM,
            profile,
            key_id: &key_id,
            public_key_hex: &public_key_hex,
            witness_hash: &witness_hash,
            stream_root: &stream_root,
            content_hash: &content_hash,
        };
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|error| SignatureError::new("signature_payload", error.to_string()))?;
        let signature = signing_key.sign(&payload_bytes);

        Ok(Self {
            signature_version: HTP_SIGNATURE_VERSION.to_string(),
            algorithm: HTP_SIGNATURE_ALGORITHM.to_string(),
            profile,
            key_id,
            public_key_hex,
            witness_hash,
            stream_root,
            content_hash,
            signed_payload_hex: encode_hex(&payload_bytes),
            signature_hex: encode_hex(&signature.to_bytes()),
            content,
        })
    }

    pub fn verify(&self) -> SignatureVerification {
        let mut issues = Vec::new();
        if self.signature_version != HTP_SIGNATURE_VERSION {
            issues.push(format!(
                "unsupported signature version {}",
                self.signature_version
            ));
        }
        if self.algorithm != HTP_SIGNATURE_ALGORITHM {
            issues.push(format!(
                "unsupported signature algorithm {}",
                self.algorithm
            ));
        }

        let actual_content_hash = hash_json(&self.content);
        if actual_content_hash != self.content_hash {
            issues.push("content hash mismatch".to_string());
        }
        self.verify_content_binding(&mut issues);

        let expected_payload = SignaturePayload {
            signature_version: &self.signature_version,
            algorithm: &self.algorithm,
            profile: self.profile,
            key_id: &self.key_id,
            public_key_hex: &self.public_key_hex,
            witness_hash: &self.witness_hash,
            stream_root: &self.stream_root,
            content_hash: &self.content_hash,
        };
        let expected_payload_bytes = match serde_json::to_vec(&expected_payload) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!("cannot encode signature payload: {error}"));
                Vec::new()
            }
        };
        if !expected_payload_bytes.is_empty()
            && encode_hex(&expected_payload_bytes) != self.signed_payload_hex
        {
            issues.push("signed payload does not match envelope metadata".to_string());
        }

        let public_key = match parse_fixed_hex::<32>(&self.public_key_hex) {
            Ok(value) => VerifyingKey::from_bytes(&value).map_err(|error| error.to_string()),
            Err(error) => Err(error),
        };
        let signature = parse_fixed_hex::<64>(&self.signature_hex);
        match (public_key, signature) {
            (Ok(key), Ok(signature_bytes)) if !expected_payload_bytes.is_empty() => {
                let signature = Signature::from_bytes(&signature_bytes);
                if key
                    .verify_strict(&expected_payload_bytes, &signature)
                    .is_err()
                {
                    issues.push("ed25519 signature verification failed".to_string());
                }
            }
            (Err(error), _) => issues.push(format!("invalid public key: {error}")),
            (_, Err(error)) => issues.push(format!("invalid signature: {error}")),
            _ => {}
        }

        SignatureVerification {
            valid: issues.is_empty(),
            issues,
        }
    }

    /// True only when the envelope itself verifies and it is bound to this exact
    /// transaction witness. This is the public replacement for the former private-DDC
    /// export-bundle helper.
    pub fn matches_transaction(&self, transaction: &HumanAiTransaction) -> bool {
        self.verify().valid && self.witness_hash == transaction.witness_hash()
    }

    fn verify_content_binding(&self, issues: &mut Vec<String>) {
        match self.profile {
            PublicationProfile::Public => {
                if self.content.get("profile").and_then(Value::as_str) != Some("public") {
                    issues.push(
                        "public envelope content does not declare public profile".to_string(),
                    );
                }
                if self.content.get("protocol_version").and_then(Value::as_str)
                    != Some(HTP_LIVE_VERSION)
                {
                    issues.push("public envelope content has wrong protocol version".to_string());
                }
                if self.content.get("witness_hash").and_then(Value::as_str)
                    != Some(self.witness_hash.as_str())
                {
                    issues.push("public content witness hash does not match envelope".to_string());
                }
                if self.content.get("stream_root").and_then(Value::as_str)
                    != Some(self.stream_root.as_str())
                {
                    issues.push("public content stream root does not match envelope".to_string());
                }
            }
            PublicationProfile::Private => {
                match serde_json::from_value::<LiveWitnessSnapshot>(self.content.clone()) {
                    Ok(snapshot) => {
                        if snapshot.protocol_version != HTP_LIVE_VERSION {
                            issues.push("private content has wrong protocol version".to_string());
                        }
                        if snapshot.witness_hash() != self.witness_hash {
                            issues.push(
                                "private content witness hash does not match envelope".to_string(),
                            );
                        }
                        if snapshot.stream_root != self.stream_root {
                            issues.push(
                                "private content stream root does not match envelope".to_string(),
                            );
                        }
                    }
                    Err(error) => {
                        issues.push(format!("private content is not a live snapshot: {error}"))
                    }
                }
            }
        }
    }
}

pub fn publish_snapshot(snapshot: &LiveWitnessSnapshot, profile: PublicationProfile) -> Value {
    match profile {
        PublicationProfile::Private => {
            serde_json::to_value(snapshot).expect("LiveWitnessSnapshot serialization cannot fail")
        }
        PublicationProfile::Public => public_snapshot(snapshot),
    }
}

fn public_snapshot(snapshot: &LiveWitnessSnapshot) -> Value {
    let transaction = &snapshot.transaction;
    let evidence: Vec<Value> = transaction
        .evidence
        .iter()
        .map(|record| {
            json!({
                "id": record.id,
                "kind": record.kind,
                "statement": record.statement,
                "status": record.status,
                "source_digest": tagged_hash("evidence-source", record.source.as_bytes()),
            })
        })
        .collect();
    let actions: Vec<Value> = transaction
        .actions
        .iter()
        .map(|record| {
            let dimensions: BTreeSet<Dimension> = record
                .effects
                .iter()
                .map(|effect| effect.dimension)
                .collect();
            json!({
                "id": record.id,
                "description": record.description,
                "requires": record.requires,
                "status": record.status,
                "affected_dimensions": dimensions,
                "reversible": record.reversible,
            })
        })
        .collect();
    let receipts: Vec<Value> = snapshot.event_receipts.iter().map(public_receipt).collect();

    json!({
        "protocol_version": HTP_LIVE_VERSION,
        "profile": "public",
        "transaction_protocol_version": transaction.protocol_version,
        "conversation_id": transaction.conversation_id,
        "turn_id": transaction.turn_id,
        "predecessor_witness": transaction.predecessor_witness,
        "witness_hash": snapshot.witness_hash(),
        "stream_root": snapshot.stream_root,
        "state": transaction.state(),
        "need": {
            "objective": transaction.need.objective,
            "constraint_count": transaction.need.constraints.len(),
        },
        "projection": {
            "interpretation": transaction.projection.interpretation,
            "ambiguities": transaction.projection.ambiguities,
            "assumption_count": transaction.projection.assumptions.len(),
        },
        "authority": transaction.authority,
        "evidence": evidence,
        "actions": actions,
        "witness": transaction.witness,
        "fractures": transaction.fractures,
        "repairs": transaction.repairs,
        "change": snapshot.change,
        "event_receipts": receipts,
        "redacted": {
            "human_statement": true,
            "constraints": true,
            "projection_assumptions": true,
            "evidence_sources": "sha256_digest_only",
            "event_source_ids": true,
            "action_effect_boundaries": true,
            "resource_deltas": true,
        }
    })
}

fn public_receipt(receipt: &EventReceipt) -> Value {
    json!({
        "event_id_digest": tagged_hash("event-id", receipt.event_id.as_bytes()),
        "actor": receipt.actor,
        "event_type": receipt.event_type,
        "payload_hash": receipt.payload_hash,
        "stream_root": receipt.stream_root,
    })
}

fn hash_json<T: Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).expect("HTP security values must serialize");
    tagged_hash("json", &encoded)
}

fn tagged_hash(tag: &str, value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"ddc-htp-security-v0.2\0");
    hasher.update(tag.as_bytes());
    hasher.update(b"\0");
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn parse_fixed_hex<const N: usize>(value: &str) -> Result<[u8; N], String> {
    if value.len() != N * 2 {
        return Err(format!("expected {} hexadecimal characters", N * 2));
    }
    let mut decoded = [0u8; N];
    let bytes = value.as_bytes();
    for index in 0..N {
        let high = decode_nibble(bytes[index * 2])?;
        let low = decode_nibble(bytes[index * 2 + 1])?;
        decoded[index] = (high << 4) | low;
    }
    Ok(decoded)
}

fn decode_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("value contains a non-hexadecimal character".to_string()),
    }
}
