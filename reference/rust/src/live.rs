// SPDX-License-Identifier: Apache-2.0

use crate::dimension::{Dimension, Effect};
use crate::protocol::{
    ActionRecord, ActionStatus, AiProjection, AuthorityState, ChangeSummary, ClaimStatus,
    EvidenceKind, EvidenceRecord, FractureRecord, FractureSeverity, HumanAiTransaction, HumanNeed,
    IssueSeverity, RepairRecord, WitnessRecord, HTP_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Historical HTP 0.2 wire identifier retained for compatibility.
pub const HTP_LIVE_VERSION: &str = "ddc-htp/0.2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventActor {
    Human,
    Assistant,
    Tool,
    System,
    Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityScope {
    Turn,
    Conversation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IngressTrustPolicy {
    /// source_id -> actor identity. The event may not claim a different actor.
    #[serde(default)]
    pub sources: BTreeMap<String, EventActor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationEvent {
    pub event_id: String,
    pub source_id: String,
    pub conversation_id: String,
    pub turn_id: String,
    pub actor: EventActor,
    pub payload: ConversationEventPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationEventPayload {
    HumanMessage {
        statement: String,
        objective: String,
        #[serde(default)]
        constraints: Vec<String>,
    },
    AiProjection {
        interpretation: String,
        #[serde(default)]
        assumptions: Vec<String>,
        #[serde(default)]
        ambiguities: Vec<String>,
    },
    AuthorityRequired {
        capability: String,
    },
    AuthorityGrant {
        capability: String,
        scope: AuthorityScope,
    },
    AuthorityDeny {
        capability: String,
        scope: AuthorityScope,
    },
    AuthorityRevoke {
        capability: String,
    },
    Evidence {
        id: String,
        kind: EvidenceKind,
        statement: String,
        source: String,
        status: ClaimStatus,
    },
    Action {
        id: String,
        description: String,
        #[serde(default)]
        requires: BTreeSet<String>,
        status: ActionStatus,
        #[serde(default)]
        effects: BTreeSet<Effect>,
        #[serde(default)]
        resource_delta: BTreeMap<String, i64>,
        #[serde(default)]
        reversible: Option<bool>,
    },
    Fracture {
        id: String,
        dimension: Dimension,
        summary: String,
        severity: FractureSeverity,
        #[serde(default)]
        evidence_ids: BTreeSet<String>,
    },
    Repair {
        fracture_id: String,
        summary: String,
        #[serde(default)]
        evidence_ids: BTreeSet<String>,
    },
    AiResponse {
        conclusion: String,
        #[serde(default)]
        recommendation: Option<String>,
        #[serde(default)]
        evidence_ids: BTreeSet<String>,
        #[serde(default)]
        action_ids: BTreeSet<String>,
    },
}

impl ConversationEventPayload {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::HumanMessage { .. } => "human_message",
            Self::AiProjection { .. } => "ai_projection",
            Self::AuthorityRequired { .. } => "authority_required",
            Self::AuthorityGrant { .. } => "authority_grant",
            Self::AuthorityDeny { .. } => "authority_deny",
            Self::AuthorityRevoke { .. } => "authority_revoke",
            Self::Evidence { .. } => "evidence",
            Self::Action { .. } => "action",
            Self::Fracture { .. } => "fracture",
            Self::Repair { .. } => "repair",
            Self::AiResponse { .. } => "ai_response",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventReceipt {
    pub event_id: String,
    pub source_id: String,
    pub actor: EventActor,
    pub event_type: String,
    pub payload_hash: String,
    pub stream_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveWitnessSnapshot {
    pub protocol_version: String,
    pub transaction: HumanAiTransaction,
    pub event_receipts: Vec<EventReceipt>,
    pub stream_root: String,
    #[serde(default)]
    pub change: Option<ChangeSummary>,
}

impl LiveWitnessSnapshot {
    pub fn witness_hash(&self) -> String {
        self.transaction.witness_hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub event_id: Option<String>,
}

impl IngestError {
    fn new(code: &str, message: impl Into<String>, event: Option<&ConversationEvent>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            event_id: event.map(|item| item.event_id.clone()),
        }
    }
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(event_id) = &self.event_id {
            write!(f, "{} ({event_id}): {}", self.code, self.message)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for IngestError {}

#[derive(Debug, Clone)]
pub struct LiveConversationAssembler {
    policy: IngressTrustPolicy,
    conversation_id: Option<String>,
    active: Option<HumanAiTransaction>,
    previous: Option<HumanAiTransaction>,
    active_receipts: Vec<EventReceipt>,
    stream_root: String,
    seen_event_ids: BTreeSet<String>,
    conversation_grants: BTreeSet<String>,
    conversation_denials: BTreeSet<String>,
}

impl LiveConversationAssembler {
    pub fn new(policy: IngressTrustPolicy) -> Self {
        Self {
            policy,
            conversation_id: None,
            active: None,
            previous: None,
            active_receipts: Vec::new(),
            stream_root: genesis_stream_root(),
            seen_event_ids: BTreeSet::new(),
            conversation_grants: BTreeSet::new(),
            conversation_denials: BTreeSet::new(),
        }
    }

    pub fn stream_root(&self) -> &str {
        &self.stream_root
    }

    pub fn previous(&self) -> Option<&HumanAiTransaction> {
        self.previous.as_ref()
    }

    pub fn has_active_turn(&self) -> bool {
        self.active.is_some()
    }

    pub fn ingest(&mut self, event: ConversationEvent) -> Result<Option<LiveWitnessSnapshot>, IngestError> {
        self.preflight(&event)?;

        let payload_hash = hash_json(&event.payload);
        let next_root = chain_stream_root(&self.stream_root, &hash_json(&event));
        let receipt = EventReceipt {
            event_id: event.event_id.clone(),
            source_id: event.source_id.clone(),
            actor: event.actor,
            event_type: event.payload.event_type().to_string(),
            payload_hash,
            stream_root: next_root.clone(),
        };

        let mut snapshot = self.apply(&event)?;
        self.seen_event_ids.insert(event.event_id.clone());
        self.stream_root = next_root;
        self.active_receipts.push(receipt);

        if let Some(mut completed) = snapshot.take() {
            completed.event_receipts = std::mem::take(&mut self.active_receipts);
            completed.stream_root = self.stream_root.clone();
            self.previous = Some(completed.transaction.clone());
            return Ok(Some(completed));
        }

        Ok(None)
    }

    fn preflight(&self, event: &ConversationEvent) -> Result<(), IngestError> {
        if event.event_id.trim().is_empty() {
            return Err(IngestError::new("missing_event_id", "event_id must not be empty", Some(event)));
        }
        if self.seen_event_ids.contains(&event.event_id) {
            return Err(IngestError::new("event_replay", "event_id has already been accepted", Some(event)));
        }
        if event.source_id.trim().is_empty() {
            return Err(IngestError::new("missing_source_id", "source_id must not be empty", Some(event)));
        }
        match self.policy.sources.get(&event.source_id) {
            Some(actor) if actor == &event.actor => {}
            Some(actor) => {
                return Err(IngestError::new(
                    "source_actor_mismatch",
                    format!("trusted source {} is registered as {:?}, not {:?}", event.source_id, actor, event.actor),
                    Some(event),
                ));
            }
            None => {
                return Err(IngestError::new(
                    "untrusted_source",
                    format!("source {} is not present in the ingress trust policy", event.source_id),
                    Some(event),
                ));
            }
        }

        if let Some(conversation_id) = &self.conversation_id {
            if conversation_id != &event.conversation_id {
                return Err(IngestError::new(
                    "conversation_mismatch",
                    format!("assembler is bound to conversation {conversation_id}, got {}", event.conversation_id),
                    Some(event),
                ));
            }
        }

        let is_start = matches!(event.payload, ConversationEventPayload::HumanMessage { .. });
        if is_start {
            if self.active.is_some() {
                return Err(IngestError::new(
                    "incomplete_turn_replaced",
                    "a new human message cannot replace an unfinished turn",
                    Some(event),
                ));
            }
        } else {
            let active = self.active.as_ref().ok_or_else(|| {
                IngestError::new(
                    "no_active_turn",
                    "event requires an active turn started by a human_message",
                    Some(event),
                )
            })?;
            if active.turn_id != event.turn_id {
                return Err(IngestError::new(
                    "turn_mismatch",
                    format!("active turn is {}, got {}", active.turn_id, event.turn_id),
                    Some(event),
                ));
            }
        }

        self.validate_actor_for_payload(event)?;
        self.validate_references(event)?;
        Ok(())
    }

    fn validate_actor_for_payload(&self, event: &ConversationEvent) -> Result<(), IngestError> {
        let valid = match &event.payload {
            ConversationEventPayload::HumanMessage { .. } => event.actor == EventActor::Human,
            ConversationEventPayload::AiProjection { .. }
            | ConversationEventPayload::AiResponse { .. } => event.actor == EventActor::Assistant,
            ConversationEventPayload::AuthorityRequired { .. } => matches!(
                event.actor,
                EventActor::Assistant | EventActor::System | EventActor::Policy
            ),
            ConversationEventPayload::AuthorityGrant { .. }
            | ConversationEventPayload::AuthorityDeny { .. }
            | ConversationEventPayload::AuthorityRevoke { .. } => {
                matches!(event.actor, EventActor::Human | EventActor::Policy)
            }
            ConversationEventPayload::Evidence { kind, .. } => match kind {
                EvidenceKind::UserStatement => event.actor == EventActor::Human,
                EvidenceKind::ToolResult => event.actor == EventActor::Tool,
                EvidenceKind::SystemObservation => event.actor == EventActor::System,
                EvidenceKind::Source => matches!(event.actor, EventActor::Tool | EventActor::System),
                EvidenceKind::Calculation => matches!(
                    event.actor,
                    EventActor::Assistant | EventActor::Tool | EventActor::System
                ),
            },
            ConversationEventPayload::Action { status, .. } => match status {
                ActionStatus::Proposed | ActionStatus::Skipped => matches!(
                    event.actor,
                    EventActor::Assistant | EventActor::System | EventActor::Tool
                ),
                ActionStatus::Authorized => matches!(
                    event.actor,
                    EventActor::Human | EventActor::Policy | EventActor::System
                ),
                ActionStatus::Executed | ActionStatus::Failed => {
                    matches!(event.actor, EventActor::Tool | EventActor::System)
                }
            },
            ConversationEventPayload::Fracture { .. } => matches!(
                event.actor,
                EventActor::Assistant | EventActor::Tool | EventActor::System | EventActor::Policy
            ),
            ConversationEventPayload::Repair { .. } => true,
        };

        if valid {
            return Ok(());
        }

        let code = match event.payload {
            ConversationEventPayload::AuthorityGrant { .. }
            | ConversationEventPayload::AuthorityDeny { .. }
            | ConversationEventPayload::AuthorityRevoke { .. } => "authority_escalation_origin",
            ConversationEventPayload::Evidence { .. } => "fabricated_evidence_origin",
            ConversationEventPayload::Action { .. } => "action_origin_mismatch",
            _ => "event_actor_mismatch",
        };
        Err(IngestError::new(
            code,
            format!("actor {:?} may not emit {} events", event.actor, event.payload.event_type()),
            Some(event),
        ))
    }

    fn validate_references(&self, event: &ConversationEvent) -> Result<(), IngestError> {
        let Some(active) = self.active.as_ref() else {
            return Ok(());
        };

        match &event.payload {
            ConversationEventPayload::Evidence { id, .. } => {
                if active.evidence.iter().any(|item| &item.id == id) {
                    return Err(IngestError::new(
                        "duplicate_evidence_id",
                        format!("evidence id {id} already exists in this turn"),
                        Some(event),
                    ));
                }
            }
            ConversationEventPayload::Action { id, .. } => {
                if active.actions.iter().any(|item| &item.id == id) {
                    return Err(IngestError::new(
                        "duplicate_action_id",
                        format!("action id {id} already exists in this turn"),
                        Some(event),
                    ));
                }
            }
            ConversationEventPayload::Fracture { id, evidence_ids, .. } => {
                if active.fractures.iter().any(|item| &item.id == id) {
                    return Err(IngestError::new(
                        "duplicate_fracture_id",
                        format!("fracture id {id} already exists in this turn"),
                        Some(event),
                    ));
                }
                self.ensure_evidence_exists(active, evidence_ids, event)?;
            }
            ConversationEventPayload::Repair { fracture_id, evidence_ids, .. } => {
                if !active.fractures.iter().any(|item| &item.id == fracture_id) {
                    return Err(IngestError::new(
                        "repair_without_fracture",
                        format!("repair references unknown fracture {fracture_id}"),
                        Some(event),
                    ));
                }
                self.ensure_evidence_exists(active, evidence_ids, event)?;
            }
            ConversationEventPayload::AiResponse { evidence_ids, action_ids, .. } => {
                self.ensure_evidence_exists(active, evidence_ids, event)?;
                let known_actions: BTreeSet<_> = active.actions.iter().map(|item| item.id.as_str()).collect();
                for id in action_ids {
                    if !known_actions.contains(id.as_str()) {
                        return Err(IngestError::new(
                            "unknown_response_action",
                            format!("AI response references unknown action {id}"),
                            Some(event),
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn ensure_evidence_exists(
        &self,
        active: &HumanAiTransaction,
        ids: &BTreeSet<String>,
        event: &ConversationEvent,
    ) -> Result<(), IngestError> {
        let known: BTreeSet<_> = active.evidence.iter().map(|item| item.id.as_str()).collect();
        for id in ids {
            if !known.contains(id.as_str()) {
                return Err(IngestError::new(
                    "unknown_event_evidence",
                    format!("event references unknown evidence {id}"),
                    Some(event),
                ));
            }
        }
        Ok(())
    }

    fn apply(&mut self, event: &ConversationEvent) -> Result<Option<LiveWitnessSnapshot>, IngestError> {
        if let ConversationEventPayload::HumanMessage { statement, objective, constraints } = &event.payload {
            self.conversation_id.get_or_insert_with(|| event.conversation_id.clone());
            let authority = AuthorityState {
                required: BTreeSet::new(),
                granted: self.conversation_grants.clone(),
                denied: self.conversation_denials.clone(),
            };
            self.active = Some(HumanAiTransaction {
                protocol_version: HTP_VERSION.to_string(),
                conversation_id: event.conversation_id.clone(),
                turn_id: event.turn_id.clone(),
                predecessor_witness: self.previous.as_ref().map(HumanAiTransaction::witness_hash),
                need: HumanNeed {
                    statement: statement.clone(),
                    objective: objective.clone(),
                    constraints: constraints.clone(),
                },
                projection: AiProjection::default(),
                authority,
                evidence: Vec::new(),
                actions: Vec::new(),
                witness: WitnessRecord::default(),
                fractures: Vec::new(),
                repairs: Vec::new(),
            });
            return Ok(None);
        }

        let active = self.active.as_mut().expect("preflight requires active turn");
        match &event.payload {
            ConversationEventPayload::AiProjection { interpretation, assumptions, ambiguities } => {
                active.projection = AiProjection {
                    interpretation: interpretation.clone(),
                    assumptions: assumptions.clone(),
                    ambiguities: ambiguities.clone(),
                };
            }
            ConversationEventPayload::AuthorityRequired { capability } => {
                active.authority.required.insert(capability.clone());
            }
            ConversationEventPayload::AuthorityGrant { capability, scope } => {
                active.authority.denied.remove(capability);
                active.authority.granted.insert(capability.clone());
                if *scope == AuthorityScope::Conversation {
                    self.conversation_denials.remove(capability);
                    self.conversation_grants.insert(capability.clone());
                }
            }
            ConversationEventPayload::AuthorityDeny { capability, scope } => {
                active.authority.granted.remove(capability);
                active.authority.denied.insert(capability.clone());
                if *scope == AuthorityScope::Conversation {
                    self.conversation_grants.remove(capability);
                    self.conversation_denials.insert(capability.clone());
                }
            }
            ConversationEventPayload::AuthorityRevoke { capability } => {
                active.authority.granted.remove(capability);
                self.conversation_grants.remove(capability);
            }
            ConversationEventPayload::Evidence { id, kind, statement, source, status } => {
                active.evidence.push(EvidenceRecord {
                    id: id.clone(),
                    kind: *kind,
                    statement: statement.clone(),
                    source: source.clone(),
                    status: *status,
                });
            }
            ConversationEventPayload::Action {
                id,
                description,
                requires,
                status,
                effects,
                resource_delta,
                reversible,
            } => {
                active.actions.push(ActionRecord {
                    id: id.clone(),
                    description: description.clone(),
                    requires: requires.clone(),
                    status: *status,
                    effects: effects.clone(),
                    resource_delta: resource_delta.clone(),
                    reversible: *reversible,
                });
            }
            ConversationEventPayload::Fracture { id, dimension, summary, severity, evidence_ids } => {
                active.fractures.push(FractureRecord {
                    id: id.clone(),
                    dimension: *dimension,
                    summary: summary.clone(),
                    severity: *severity,
                    evidence_ids: evidence_ids.clone(),
                });
            }
            ConversationEventPayload::Repair { fracture_id, summary, evidence_ids } => {
                active.repairs.push(RepairRecord {
                    fracture_id: fracture_id.clone(),
                    summary: summary.clone(),
                    evidence_ids: evidence_ids.clone(),
                });
            }
            ConversationEventPayload::AiResponse { conclusion, recommendation, evidence_ids, action_ids } => {
                active.witness = WitnessRecord {
                    conclusion: conclusion.clone(),
                    recommendation: recommendation.clone(),
                    evidence_ids: evidence_ids.clone(),
                    action_ids: action_ids.clone(),
                };
                let report = active.validate();
                if !report.valid {
                    let details = report
                        .issues
                        .iter()
                        .filter(|issue| issue.severity == IssueSeverity::Error)
                        .map(|issue| format!("{}: {}", issue.code, issue.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Err(IngestError::new("invalid_completed_transaction", details, Some(event)));
                }
                let transaction = self.active.take().expect("active transaction exists");
                let change = self.previous.as_ref().map(|previous| transaction.diff_from(previous));
                return Ok(Some(LiveWitnessSnapshot {
                    protocol_version: HTP_LIVE_VERSION.to_string(),
                    transaction,
                    event_receipts: Vec::new(),
                    stream_root: String::new(),
                    change,
                }));
            }
            ConversationEventPayload::HumanMessage { .. } => unreachable!("handled above"),
        }
        Ok(None)
    }
}

fn genesis_stream_root() -> String {
    hash_bytes(b"ddc-htp-live-stream-genesis-v0.2")
}

fn chain_stream_root(previous: &str, event_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"ddc-htp-live-stream-v0.2\0");
    hasher.update(previous.as_bytes());
    hasher.update(b"\0");
    hasher.update(event_hash.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hash_json<T: Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).expect("HTP live values must serialize");
    hash_bytes(&encoded)
}

fn hash_bytes(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}
