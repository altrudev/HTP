// SPDX-License-Identifier: Apache-2.0

use htp_reference::{
    ActionRecord, ActionStatus, AiProjection, AuthorityState, ClaimStatus, Dimension,
    EvidenceKind, EvidenceRecord, FractureRecord, FractureSeverity, HumanAiTransaction, HumanNeed,
    ProtocolState, RepairRecord, TranslationLevel, WitnessRecord, HTP_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

fn base_transaction() -> HumanAiTransaction {
    HumanAiTransaction {
        protocol_version: HTP_VERSION.to_string(),
        conversation_id: "conv-update".to_string(),
        turn_id: "turn-1".to_string(),
        predecessor_witness: None,
        need: HumanNeed {
            statement: "Can I safely install this update?".to_string(),
            objective: "Decide whether installing the update creates unacceptable risk".to_string(),
            constraints: vec!["Do not change the system without permission".to_string()],
        },
        projection: AiProjection {
            interpretation: "Evaluate compatibility, security, and rollback risk".to_string(),
            assumptions: vec!["The installed operating system is Ubuntu 24.04".to_string()],
            ambiguities: vec![],
        },
        authority: AuthorityState::default(),
        evidence: vec![EvidenceRecord {
            id: "e-os".to_string(),
            kind: EvidenceKind::SystemObservation,
            statement: "The system reports Ubuntu 24.04".to_string(),
            source: "local system inventory".to_string(),
            status: ClaimStatus::Established,
        }],
        actions: vec![],
        witness: WitnessRecord {
            conclusion: "The update appears compatible with the base operating system".to_string(),
            recommendation: Some("Verify the third-party plugin before installing".to_string()),
            evidence_ids: set(&["e-os"]),
            action_ids: BTreeSet::new(),
        },
        fractures: vec![],
        repairs: vec![],
    }
}

#[test]
fn plain_projection_separates_need_interpretation_and_unknowns() {
    let mut transaction = base_transaction();
    transaction.evidence.push(EvidenceRecord {
        id: "e-plugin".to_string(),
        kind: EvidenceKind::Source,
        statement: "Plugin compatibility has not yet been verified".to_string(),
        source: "plugin documentation not yet checked".to_string(),
        status: ClaimStatus::Unknown,
    });

    let projection = transaction.project(TranslationLevel::Plain, None);
    assert_eq!(projection["you_want"], "Decide whether installing the update creates unacceptable risk");
    assert_eq!(projection["ai_understood"], "Evaluate compatibility, security, and rollback risk");
    assert_eq!(projection["unknowns"][0], "Plugin compatibility has not yet been verified");
}

#[test]
fn executed_action_without_authority_is_invalid() {
    let mut transaction = base_transaction();
    transaction.authority.required = set(&["system.install"]);
    transaction.actions.push(ActionRecord {
        id: "a-install".to_string(),
        description: "Install the update".to_string(),
        requires: set(&["system.install"]),
        status: ActionStatus::Executed,
        effects: BTreeSet::new(),
        resource_delta: BTreeMap::new(),
        reversible: Some(true),
    });
    transaction.witness.action_ids.insert("a-install".to_string());

    let report = transaction.validate();
    assert!(!report.valid);
    assert!(report.issues.iter().any(|issue| issue.code == "action_without_authority"));
    assert_eq!(transaction.state(), ProtocolState::AwaitingAuthority);
}

#[test]
fn blocking_fracture_closes_only_after_repair() {
    let mut transaction = base_transaction();
    transaction.fractures.push(FractureRecord {
        id: "f-plugin".to_string(),
        dimension: Dimension::Semantic,
        summary: "Plugin compatibility is unresolved".to_string(),
        severity: FractureSeverity::Blocking,
        evidence_ids: BTreeSet::new(),
    });
    assert_eq!(transaction.state(), ProtocolState::Fractured);

    transaction.repairs.push(RepairRecord {
        fracture_id: "f-plugin".to_string(),
        summary: "Plugin compatibility was verified against current documentation".to_string(),
        evidence_ids: BTreeSet::new(),
    });
    assert_eq!(transaction.state(), ProtocolState::Closed);
}

#[test]
fn changed_answer_produces_portable_dimensional_change() {
    let previous = base_transaction();
    let mut current = previous.clone();
    current.turn_id = "turn-2".to_string();
    current.predecessor_witness = Some(previous.witness_hash());
    current.evidence.push(EvidenceRecord {
        id: "e-plugin-doc".to_string(),
        kind: EvidenceKind::Source,
        statement: "Plugin X supports only versions through 3.7".to_string(),
        source: "Plugin X compatibility documentation".to_string(),
        status: ClaimStatus::Established,
    });
    current.witness.evidence_ids.insert("e-plugin-doc".to_string());
    current.witness.conclusion = "Do not install yet because Plugin X is not compatible with the new version".to_string();
    current.witness.recommendation = Some("Wait for a compatible Plugin X release".to_string());

    let dimensional = current.dimensional_change_from(&previous);
    assert!(dimensional.changed_dimensions.contains_key(&Dimension::Semantic));
    assert!(dimensional.changed_dimensions.contains_key(&Dimension::Frequency));
    assert!(dimensional.changed_dimensions.contains_key(&Dimension::Lineage));
    assert!(dimensional.conserved_dimensions.contains(&Dimension::Authority));
    assert!(dimensional.conserved_dimensions.contains(&Dimension::Resource));

    let change = current.diff_from(&previous);
    assert!(change.conclusion_changed);
    assert!(change.new_evidence.contains("e-plugin-doc"));
}

#[test]
fn witness_hash_is_deterministic() {
    let transaction = base_transaction();
    assert_eq!(transaction.witness_hash(), transaction.clone().witness_hash());
}
