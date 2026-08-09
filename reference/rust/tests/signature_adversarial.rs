// SPDX-License-Identifier: Apache-2.0

use htp_reference::{
    ConversationEvent, ConversationEventPayload, EventActor, IngressTrustPolicy,
    LiveConversationAssembler, PublicationProfile, SignedWitnessEnvelope,
};
use std::collections::{BTreeMap, BTreeSet};

const SECRET: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

fn snapshot() -> htp_reference::LiveWitnessSnapshot {
    let mut assembler = LiveConversationAssembler::new(IngressTrustPolicy {
        sources: BTreeMap::from([
            ("human".to_string(), EventActor::Human),
            ("assistant".to_string(), EventActor::Assistant),
        ]),
    });
    assembler
        .ingest(ConversationEvent {
            event_id: "e1".to_string(),
            source_id: "human".to_string(),
            conversation_id: "signed-binding".to_string(),
            turn_id: "turn-1".to_string(),
            actor: EventActor::Human,
            payload: ConversationEventPayload::HumanMessage {
                statement: "Check this claim".to_string(),
                objective: "Produce a bounded witness".to_string(),
                constraints: Vec::new(),
            },
        })
        .unwrap();
    assembler
        .ingest(ConversationEvent {
            event_id: "e2".to_string(),
            source_id: "assistant".to_string(),
            conversation_id: "signed-binding".to_string(),
            turn_id: "turn-1".to_string(),
            actor: EventActor::Assistant,
            payload: ConversationEventPayload::AiProjection {
                interpretation: "Answer only from recorded evidence".to_string(),
                assumptions: Vec::new(),
                ambiguities: Vec::new(),
            },
        })
        .unwrap();
    assembler
        .ingest(ConversationEvent {
            event_id: "e3".to_string(),
            source_id: "assistant".to_string(),
            conversation_id: "signed-binding".to_string(),
            turn_id: "turn-1".to_string(),
            actor: EventActor::Assistant,
            payload: ConversationEventPayload::AiResponse {
                conclusion: "No external evidence was recorded".to_string(),
                recommendation: None,
                evidence_ids: BTreeSet::new(),
                action_ids: BTreeSet::new(),
            },
        })
        .unwrap()
        .expect("response closes turn")
}

#[test]
fn public_content_binding_rejects_mismatched_stream_root() {
    let snapshot = snapshot();
    let mut envelope =
        SignedWitnessEnvelope::sign(&snapshot, PublicationProfile::Public, "test-key", SECRET)
            .unwrap();
    envelope.content["stream_root"] = serde_json::json!("0".repeat(64));
    let report = envelope.verify();
    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("stream root does not match envelope")));
}

#[test]
fn valid_envelope_cannot_be_reused_for_another_transaction() {
    let first = snapshot();
    let envelope =
        SignedWitnessEnvelope::sign(&first, PublicationProfile::Private, "test-key", SECRET)
            .unwrap();

    let mut other = first.transaction.clone();
    other.turn_id = "different-turn".to_string();
    other.witness.conclusion = "Different conclusion".to_string();

    assert!(!envelope.matches_transaction(&other));
    assert!(envelope.matches_transaction(&first.transaction));
}
