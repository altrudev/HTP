// SPDX-License-Identifier: Apache-2.0

use htp_reference::{
    ActionStatus, AuthorityScope, ClaimStatus, ConversationEvent, ConversationEventPayload,
    EventActor, EvidenceKind, IngressTrustPolicy, LiveConversationAssembler, PublicationProfile,
    SignedWitnessEnvelope,
};
use std::collections::{BTreeMap, BTreeSet};

fn policy() -> IngressTrustPolicy {
    IngressTrustPolicy {
        sources: BTreeMap::from([
            ("human-ui".to_string(), EventActor::Human),
            ("assistant".to_string(), EventActor::Assistant),
            ("tool:web".to_string(), EventActor::Tool),
            ("system".to_string(), EventActor::System),
            ("policy".to_string(), EventActor::Policy),
        ]),
    }
}

fn event(
    id: &str,
    source: &str,
    actor: EventActor,
    turn: &str,
    payload: ConversationEventPayload,
) -> ConversationEvent {
    ConversationEvent {
        event_id: id.to_string(),
        source_id: source.to_string(),
        conversation_id: "conv-live".to_string(),
        turn_id: turn.to_string(),
        actor,
        payload,
    }
}

fn human_start(id: &str, turn: &str) -> ConversationEvent {
    event(
        id,
        "human-ui",
        EventActor::Human,
        turn,
        ConversationEventPayload::HumanMessage {
            statement: "Can I install the update? My internal host is workstation-7.".to_string(),
            objective: "Decide whether the update is safe".to_string(),
            constraints: vec!["Do not reveal workstation-7 publicly".to_string()],
        },
    )
}

fn projection(id: &str, turn: &str) -> ConversationEvent {
    event(
        id,
        "assistant",
        EventActor::Assistant,
        turn,
        ConversationEventPayload::AiProjection {
            interpretation: "Check compatibility and authority before changing anything"
                .to_string(),
            assumptions: vec!["System inventory is current".to_string()],
            ambiguities: Vec::new(),
        },
    )
}

fn response(id: &str, turn: &str, evidence_ids: &[&str]) -> ConversationEvent {
    event(
        id,
        "assistant",
        EventActor::Assistant,
        turn,
        ConversationEventPayload::AiResponse {
            conclusion: "The update is compatible with the observed system".to_string(),
            recommendation: Some("Install only after explicit authority is granted".to_string()),
            evidence_ids: evidence_ids
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            action_ids: BTreeSet::new(),
        },
    )
}

#[test]
fn source_cannot_impersonate_another_actor() {
    let mut assembler = LiveConversationAssembler::new(policy());
    let spoof = event(
        "e1",
        "assistant",
        EventActor::Tool,
        "turn-1",
        ConversationEventPayload::HumanMessage {
            statement: "hello".to_string(),
            objective: "test".to_string(),
            constraints: Vec::new(),
        },
    );
    let error = assembler.ingest(spoof).expect_err("spoof must be rejected");
    assert_eq!(error.code, "source_actor_mismatch");
}

#[test]
fn assistant_cannot_grant_itself_authority() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    let escalation = event(
        "e3",
        "assistant",
        EventActor::Assistant,
        "turn-1",
        ConversationEventPayload::AuthorityGrant {
            capability: "system.install".to_string(),
            scope: AuthorityScope::Conversation,
        },
    );
    let error = assembler
        .ingest(escalation)
        .expect_err("assistant self-grant must be rejected");
    assert_eq!(error.code, "authority_escalation_origin");
}

#[test]
fn assistant_cannot_fabricate_a_tool_result() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    let fake = event(
        "e3",
        "assistant",
        EventActor::Assistant,
        "turn-1",
        ConversationEventPayload::Evidence {
            id: "tool-proof".to_string(),
            kind: EvidenceKind::ToolResult,
            statement: "The tool says the package is safe".to_string(),
            source: "invented tool output".to_string(),
            status: ClaimStatus::Established,
        },
    );
    let error = assembler
        .ingest(fake)
        .expect_err("assistant tool fabrication must be rejected");
    assert_eq!(error.code, "fabricated_evidence_origin");
}

#[test]
fn assistant_cannot_claim_effectful_execution() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    let fake_execution = event(
        "e3",
        "assistant",
        EventActor::Assistant,
        "turn-1",
        ConversationEventPayload::Action {
            id: "install".to_string(),
            description: "Install update".to_string(),
            requires: BTreeSet::from(["system.install".to_string()]),
            status: ActionStatus::Executed,
            effects: BTreeSet::new(),
            resource_delta: BTreeMap::new(),
            reversible: Some(true),
        },
    );
    let error = assembler
        .ingest(fake_execution)
        .expect_err("assistant execution spoof must be rejected");
    assert_eq!(error.code, "action_origin_mismatch");
}

#[test]
fn accepted_events_are_replay_protected() {
    let mut assembler = LiveConversationAssembler::new(policy());
    let start = human_start("e1", "turn-1");
    assembler.ingest(start.clone()).unwrap();
    let error = assembler
        .ingest(start)
        .expect_err("replay must be rejected");
    assert_eq!(error.code, "event_replay");
}

#[test]
fn live_turns_auto_link_and_conversation_authority_persists() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    assembler
        .ingest(event(
            "e3",
            "assistant",
            EventActor::Assistant,
            "turn-1",
            ConversationEventPayload::AuthorityRequired {
                capability: "system.install".to_string(),
            },
        ))
        .unwrap();
    assembler
        .ingest(event(
            "e4",
            "human-ui",
            EventActor::Human,
            "turn-1",
            ConversationEventPayload::AuthorityGrant {
                capability: "system.install".to_string(),
                scope: AuthorityScope::Conversation,
            },
        ))
        .unwrap();
    assembler
        .ingest(event(
            "e5",
            "system",
            EventActor::System,
            "turn-1",
            ConversationEventPayload::Evidence {
                id: "sys".to_string(),
                kind: EvidenceKind::SystemObservation,
                statement: "Ubuntu 24.04 is installed".to_string(),
                source: "workstation-7 local inventory".to_string(),
                status: ClaimStatus::Established,
            },
        ))
        .unwrap();
    let first = assembler
        .ingest(response("e6", "turn-1", &["sys"]))
        .unwrap()
        .expect("first turn closes");
    assert!(first.transaction.predecessor_witness.is_none());
    assert!(first
        .transaction
        .authority
        .granted
        .contains("system.install"));

    assembler.ingest(human_start("e7", "turn-2")).unwrap();
    assembler.ingest(projection("e8", "turn-2")).unwrap();
    let second = assembler
        .ingest(response("e9", "turn-2", &[]))
        .unwrap()
        .expect("second turn closes");
    assert_eq!(
        second.transaction.predecessor_witness.as_deref(),
        Some(first.witness_hash().as_str())
    );
    assert!(second
        .transaction
        .authority
        .granted
        .contains("system.install"));
    assert_ne!(first.stream_root, second.stream_root);
}

#[test]
fn signed_public_witness_verifies_and_redacts_private_material() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    assembler
        .ingest(event(
            "e3",
            "system",
            EventActor::System,
            "turn-1",
            ConversationEventPayload::Evidence {
                id: "sys".to_string(),
                kind: EvidenceKind::SystemObservation,
                statement: "Ubuntu 24.04 is installed".to_string(),
                source: "workstation-7 local inventory".to_string(),
                status: ClaimStatus::Established,
            },
        ))
        .unwrap();
    let snapshot = assembler
        .ingest(response("e4", "turn-1", &["sys"]))
        .unwrap()
        .expect("turn closes");

    let secret = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let envelope =
        SignedWitnessEnvelope::sign(&snapshot, PublicationProfile::Public, "test-key", secret)
            .unwrap();
    assert!(envelope.verify().valid);
    assert!(envelope.content["need"].get("statement").is_none());
    assert!(envelope.content["evidence"][0].get("source").is_none());
    assert!(envelope.content["evidence"][0]
        .get("source_digest")
        .is_some());
    let serialized = serde_json::to_string(&envelope.content).unwrap();
    assert!(!serialized.contains("workstation-7"));
    assert!(!serialized.contains("Do not reveal workstation-7 publicly"));

    let private =
        SignedWitnessEnvelope::sign(&snapshot, PublicationProfile::Private, "test-key", secret)
            .unwrap();
    let private_serialized = serde_json::to_string(&private.content).unwrap();
    assert!(private_serialized.contains("workstation-7"));
}

#[test]
fn signature_tampering_is_detected() {
    let mut assembler = LiveConversationAssembler::new(policy());
    assembler.ingest(human_start("e1", "turn-1")).unwrap();
    assembler.ingest(projection("e2", "turn-1")).unwrap();
    let snapshot = assembler
        .ingest(response("e3", "turn-1", &[]))
        .unwrap()
        .expect("turn closes");
    let secret = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let mut envelope =
        SignedWitnessEnvelope::sign(&snapshot, PublicationProfile::Public, "test-key", secret)
            .unwrap();
    envelope.content["witness"]["conclusion"] = serde_json::json!("tampered");
    let report = envelope.verify();
    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("content hash mismatch")));
}
