// SPDX-License-Identifier: Apache-2.0

use crate::dimension::{Dimension, DimensionalChange, Effect};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Historical HTP 0.1 wire identifier. It remains frozen for compatibility with
/// witnesses produced before HTP became an independent repository.
pub const HTP_VERSION: &str = "ddc-htp/0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Established,
    Inferred,
    Unknown,
    Contradicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    UserStatement,
    Source,
    ToolResult,
    SystemObservation,
    Calculation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Proposed,
    Authorized,
    Executed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FractureSeverity {
    Advisory,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolState {
    Open,
    AwaitingAuthority,
    Blocked,
    Fractured,
    Closed,
}

impl fmt::Display for ProtocolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ProtocolState::Open => "open",
            ProtocolState::AwaitingAuthority => "awaiting_authority",
            ProtocolState::Blocked => "blocked",
            ProtocolState::Fractured => "fractured",
            ProtocolState::Closed => "closed",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationLevel {
    Plain,
    Informed,
    Expert,
    Machine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HumanNeed {
    pub statement: String,
    pub objective: String,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AiProjection {
    pub interpretation: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub ambiguities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthorityState {
    #[serde(default)]
    pub required: BTreeSet<String>,
    #[serde(default)]
    pub granted: BTreeSet<String>,
    #[serde(default)]
    pub denied: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub kind: EvidenceKind,
    pub statement: String,
    pub source: String,
    pub status: ClaimStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub requires: BTreeSet<String>,
    pub status: ActionStatus,
    #[serde(default)]
    pub effects: BTreeSet<Effect>,
    #[serde(default)]
    pub resource_delta: BTreeMap<String, i64>,
    #[serde(default)]
    pub reversible: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WitnessRecord {
    pub conclusion: String,
    #[serde(default)]
    pub recommendation: Option<String>,
    #[serde(default)]
    pub evidence_ids: BTreeSet<String>,
    #[serde(default)]
    pub action_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FractureRecord {
    pub id: String,
    pub dimension: Dimension,
    pub summary: String,
    pub severity: FractureSeverity,
    #[serde(default)]
    pub evidence_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairRecord {
    pub fracture_id: String,
    pub summary: String,
    #[serde(default)]
    pub evidence_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanAiTransaction {
    pub protocol_version: String,
    pub conversation_id: String,
    pub turn_id: String,
    #[serde(default)]
    pub predecessor_witness: Option<String>,
    pub need: HumanNeed,
    pub projection: AiProjection,
    #[serde(default)]
    pub authority: AuthorityState,
    #[serde(default)]
    pub evidence: Vec<EvidenceRecord>,
    #[serde(default)]
    pub actions: Vec<ActionRecord>,
    pub witness: WitnessRecord,
    #[serde(default)]
    pub fractures: Vec<FractureRecord>,
    #[serde(default)]
    pub repairs: Vec<RepairRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ProtocolIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub conclusion_changed: bool,
    pub previous_conclusion: String,
    pub current_conclusion: String,
    pub authority_added: BTreeSet<String>,
    pub authority_removed: BTreeSet<String>,
    pub new_evidence: BTreeSet<String>,
    pub new_fractures: BTreeSet<String>,
    pub resolved_fractures: BTreeSet<String>,
    pub changed_dimensions: BTreeMap<Dimension, BTreeSet<String>>,
    pub conserved_dimensions: BTreeSet<Dimension>,
}

impl HumanAiTransaction {
    pub fn state(&self) -> ProtocolState {
        if self.witness.conclusion.trim().is_empty() {
            return ProtocolState::Open;
        }

        let repaired = self.repaired_fracture_ids();
        if self.fractures.iter().any(|fracture| {
            fracture.severity == FractureSeverity::Blocking && !repaired.contains(&fracture.id)
        }) {
            return ProtocolState::Fractured;
        }

        if self
            .authority
            .required
            .iter()
            .any(|capability| self.authority.denied.contains(capability))
        {
            return ProtocolState::Blocked;
        }

        if !self.missing_authority().is_empty() {
            return ProtocolState::AwaitingAuthority;
        }

        ProtocolState::Closed
    }

    pub fn witness_hash(&self) -> String {
        hash_json(self)
    }

    pub fn missing_authority(&self) -> BTreeSet<String> {
        self.authority
            .required
            .difference(&self.authority.granted)
            .cloned()
            .collect()
    }

    pub fn validate(&self) -> ValidationReport {
        let mut issues = Vec::new();

        if self.protocol_version != HTP_VERSION {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                "unsupported_protocol_version",
                format!(
                    "expected protocol_version {HTP_VERSION}, got {}",
                    self.protocol_version
                ),
            );
        }
        if self.conversation_id.trim().is_empty() {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                "missing_conversation_id",
                "conversation_id must not be empty".to_string(),
            );
        }
        if self.turn_id.trim().is_empty() {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                "missing_turn_id",
                "turn_id must not be empty".to_string(),
            );
        }
        if self.need.objective.trim().is_empty() {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                "missing_objective",
                "need.objective must not be empty".to_string(),
            );
        }
        if self.projection.interpretation.trim().is_empty() {
            push_issue(
                &mut issues,
                IssueSeverity::Error,
                "missing_interpretation",
                "projection.interpretation must not be empty".to_string(),
            );
        }
        if self.witness.conclusion.trim().is_empty() {
            push_issue(
                &mut issues,
                IssueSeverity::Warning,
                "open_transaction",
                "witness.conclusion is empty; transaction remains open".to_string(),
            );
        }

        let evidence_ids = collect_unique_ids(
            self.evidence.iter().map(|record| record.id.as_str()),
            "evidence",
            &mut issues,
        );
        let action_ids = collect_unique_ids(
            self.actions.iter().map(|record| record.id.as_str()),
            "action",
            &mut issues,
        );
        let fracture_ids = collect_unique_ids(
            self.fractures.iter().map(|record| record.id.as_str()),
            "fracture",
            &mut issues,
        );

        for evidence in &self.evidence {
            if evidence.statement.trim().is_empty() {
                push_issue(
                    &mut issues,
                    IssueSeverity::Error,
                    "empty_evidence_statement",
                    format!("evidence {} has an empty statement", evidence.id),
                );
            }
            if evidence.source.trim().is_empty() {
                push_issue(
                    &mut issues,
                    IssueSeverity::Error,
                    "missing_evidence_source",
                    format!("evidence {} has no observable source", evidence.id),
                );
            }
        }

        for evidence_id in &self.witness.evidence_ids {
            if !evidence_ids.contains(evidence_id) {
                push_issue(
                    &mut issues,
                    IssueSeverity::Error,
                    "unknown_witness_evidence",
                    format!("witness references unknown evidence {evidence_id}"),
                );
            }
        }
        for action_id in &self.witness.action_ids {
            if !action_ids.contains(action_id) {
                push_issue(
                    &mut issues,
                    IssueSeverity::Error,
                    "unknown_witness_action",
                    format!("witness references unknown action {action_id}"),
                );
            }
        }

        for action in &self.actions {
            if matches!(
                action.status,
                ActionStatus::Authorized | ActionStatus::Executed
            ) {
                let missing: BTreeSet<_> = action
                    .requires
                    .difference(&self.authority.granted)
                    .cloned()
                    .collect();
                if !missing.is_empty() {
                    push_issue(
                        &mut issues,
                        IssueSeverity::Error,
                        "action_without_authority",
                        format!(
                            "action {} is {:?} without grants: {}",
                            action.id,
                            action.status,
                            join_set(&missing)
                        ),
                    );
                }
            }
        }

        for fracture in &self.fractures {
            for evidence_id in &fracture.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    push_issue(
                        &mut issues,
                        IssueSeverity::Error,
                        "unknown_fracture_evidence",
                        format!(
                            "fracture {} references unknown evidence {evidence_id}",
                            fracture.id
                        ),
                    );
                }
            }
        }

        for repair in &self.repairs {
            if !fracture_ids.contains(&repair.fracture_id) {
                push_issue(
                    &mut issues,
                    IssueSeverity::Error,
                    "repair_without_fracture",
                    format!("repair references unknown fracture {}", repair.fracture_id),
                );
            }
            for evidence_id in &repair.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    push_issue(
                        &mut issues,
                        IssueSeverity::Error,
                        "unknown_repair_evidence",
                        format!(
                            "repair for {} references unknown evidence {evidence_id}",
                            repair.fracture_id
                        ),
                    );
                }
            }
        }

        if self
            .evidence
            .iter()
            .any(|record| record.status == ClaimStatus::Contradicted)
            && self.fractures.is_empty()
        {
            push_issue(
                &mut issues,
                IssueSeverity::Warning,
                "contradiction_without_fracture",
                "contradicted evidence exists but no fracture records it".to_string(),
            );
        }

        ValidationReport {
            valid: !issues
                .iter()
                .any(|issue| issue.severity == IssueSeverity::Error),
            issues,
        }
    }

    /// Compute HTP's portable dimensional change classification.
    ///
    /// This intentionally does not invoke or reproduce the private Crystalline/DDC
    /// transaction-closure implementation. The resulting dimensions are normative HTP
    /// metadata and can be independently reproduced by any implementation.
    pub fn dimensional_change_from(&self, previous: &Self) -> DimensionalChange {
        portable_dimensional_change(previous, self)
    }

    pub fn diff_from(&self, previous: &Self) -> ChangeSummary {
        let dimensional = self.dimensional_change_from(previous);
        let previous_evidence: BTreeSet<_> = previous
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        let current_evidence: BTreeSet<_> =
            self.evidence.iter().map(|item| item.id.clone()).collect();
        let previous_fractures: BTreeSet<_> = previous
            .fractures
            .iter()
            .map(|item| item.id.clone())
            .collect();
        let current_fractures: BTreeSet<_> =
            self.fractures.iter().map(|item| item.id.clone()).collect();
        let current_repaired = self.repaired_fracture_ids();

        ChangeSummary {
            conclusion_changed: previous.witness.conclusion != self.witness.conclusion,
            previous_conclusion: previous.witness.conclusion.clone(),
            current_conclusion: self.witness.conclusion.clone(),
            authority_added: self
                .authority
                .granted
                .difference(&previous.authority.granted)
                .cloned()
                .collect(),
            authority_removed: previous
                .authority
                .granted
                .difference(&self.authority.granted)
                .cloned()
                .collect(),
            new_evidence: current_evidence
                .difference(&previous_evidence)
                .cloned()
                .collect(),
            new_fractures: current_fractures
                .difference(&previous_fractures)
                .cloned()
                .collect(),
            resolved_fractures: previous_fractures
                .iter()
                .filter(|id| !current_fractures.contains(*id) || current_repaired.contains(*id))
                .cloned()
                .collect(),
            changed_dimensions: dimensional.changed_dimensions,
            conserved_dimensions: dimensional.conserved_dimensions,
        }
    }

    pub fn project(&self, level: TranslationLevel, previous: Option<&Self>) -> Value {
        match level {
            TranslationLevel::Plain => self.project_plain(previous),
            TranslationLevel::Informed => self.project_informed(previous),
            TranslationLevel::Expert => self.project_expert(previous),
            TranslationLevel::Machine => {
                serde_json::to_value(self).expect("HumanAiTransaction serialization cannot fail")
            }
        }
    }

    fn project_plain(&self, previous: Option<&Self>) -> Value {
        let unknowns: Vec<_> = self
            .evidence
            .iter()
            .filter(|item| item.status == ClaimStatus::Unknown)
            .map(|item| item.statement.clone())
            .collect();
        let unresolved: Vec<_> = self
            .unresolved_fractures()
            .into_iter()
            .map(|fracture| fracture.summary.clone())
            .collect();

        json!({
            "state": self.state(),
            "you_want": self.need.objective,
            "ai_understood": self.projection.interpretation,
            "conclusion": self.witness.conclusion,
            "recommendation": self.witness.recommendation,
            "unknowns": unknowns,
            "authority_needed": self.missing_authority(),
            "unresolved_fractures": unresolved,
            "change": previous.map(|item| self.diff_from(item)),
        })
    }

    fn project_informed(&self, previous: Option<&Self>) -> Value {
        json!({
            "protocol_version": self.protocol_version,
            "state": self.state(),
            "need": self.need,
            "projection": self.projection,
            "authority": self.authority,
            "evidence": self.evidence,
            "actions": self.actions,
            "witness": self.witness,
            "fractures": self.fractures,
            "repairs": self.repairs,
            "change": previous.map(|item| self.diff_from(item)),
        })
    }

    fn project_expert(&self, previous: Option<&Self>) -> Value {
        json!({
            "protocol_version": self.protocol_version,
            "conversation_id": self.conversation_id,
            "turn_id": self.turn_id,
            "predecessor_witness": self.predecessor_witness,
            "witness_hash": self.witness_hash(),
            "state": self.state(),
            "validation": self.validate(),
            "dimensional_boundary": {
                "vocabulary": "ddc-eight-dimensions",
                "mode": "portable-htp",
                "private_crystalline_closure_included": false,
            },
            "change": previous.map(|item| self.diff_from(item)),
            "public": self.project_informed(previous),
        })
    }

    fn repaired_fracture_ids(&self) -> BTreeSet<String> {
        self.repairs
            .iter()
            .map(|repair| repair.fracture_id.clone())
            .collect()
    }

    fn unresolved_fractures(&self) -> Vec<&FractureRecord> {
        let repaired = self.repaired_fracture_ids();
        self.fractures
            .iter()
            .filter(|fracture| !repaired.contains(&fracture.id))
            .collect()
    }
}

fn portable_dimensional_change(
    previous: &HumanAiTransaction,
    current: &HumanAiTransaction,
) -> DimensionalChange {
    let mut changed: BTreeMap<Dimension, BTreeSet<String>> = BTreeMap::new();
    let transaction_boundary = format!("transaction:{}", current.conversation_id);
    let conversation_boundary = format!("conversation:{}", current.conversation_id);

    let previous_semantic = hash_json(&(
        &previous.need.statement,
        &previous.need.objective,
        &previous.need.constraints,
        &previous.projection,
        &previous.witness.conclusion,
        &previous.witness.recommendation,
    ));
    let current_semantic = hash_json(&(
        &current.need.statement,
        &current.need.objective,
        &current.need.constraints,
        &current.projection,
        &current.witness.conclusion,
        &current.witness.recommendation,
    ));
    if previous_semantic != current_semantic {
        changed
            .entry(Dimension::Semantic)
            .or_default()
            .extend([transaction_boundary.clone(), "need".to_string()]);
    }

    if previous.authority != current.authority {
        let mut boundaries = BTreeSet::new();
        for capability in previous
            .authority
            .required
            .union(&current.authority.required)
        {
            boundaries.insert(format!("required:{capability}"));
        }
        for capability in previous.authority.granted.union(&current.authority.granted) {
            boundaries.insert(format!("granted:{capability}"));
        }
        for capability in previous.authority.denied.union(&current.authority.denied) {
            boundaries.insert(format!("denied:{capability}"));
        }
        if boundaries.is_empty() {
            boundaries.insert(transaction_boundary.clone());
        }
        changed.insert(Dimension::Authority, boundaries);
    }

    if previous.state() != current.state() {
        changed
            .entry(Dimension::State)
            .or_default()
            .insert(conversation_boundary.clone());
    }

    let previous_resources = aggregate_resources(previous);
    let current_resources = aggregate_resources(current);
    if previous_resources != current_resources {
        let keys: BTreeSet<_> = previous_resources
            .keys()
            .chain(current_resources.keys())
            .cloned()
            .collect();
        changed.insert(
            Dimension::Resource,
            keys.into_iter()
                .map(|key| format!("resource:{key}"))
                .collect(),
        );
    }

    let previous_security = effect_boundaries(previous, Dimension::Security);
    let current_security = effect_boundaries(current, Dimension::Security);
    if previous_security != current_security {
        let boundaries = previous_security
            .union(&current_security)
            .map(|boundary| format!("dep:{boundary}"))
            .collect();
        changed.insert(Dimension::Security, boundaries);
    }

    let previous_physical = effect_boundaries(previous, Dimension::Physical);
    let current_physical = effect_boundaries(current, Dimension::Physical);
    if previous_physical != current_physical {
        changed
            .entry(Dimension::Physical)
            .or_default()
            .insert(conversation_boundary.clone());
    }

    let previous_frequency = (
        previous.evidence.len(),
        previous.actions.len(),
        previous.fractures.len(),
        previous.repairs.len(),
    );
    let current_frequency = (
        current.evidence.len(),
        current.actions.len(),
        current.fractures.len(),
        current.repairs.len(),
    );
    if previous_frequency != current_frequency {
        changed
            .entry(Dimension::Frequency)
            .or_default()
            .insert(conversation_boundary.clone());
    }

    let previous_lineage = hash_json(&(
        &previous.predecessor_witness,
        &previous.evidence,
        &previous.repairs,
        &previous.witness.evidence_ids,
        &previous.witness.action_ids,
    ));
    let current_lineage = hash_json(&(
        &current.predecessor_witness,
        &current.evidence,
        &current.repairs,
        &current.witness.evidence_ids,
        &current.witness.action_ids,
    ));
    if previous_lineage != current_lineage {
        changed
            .entry(Dimension::Lineage)
            .or_default()
            .insert(format!("lineage:{}", current.conversation_id));
    }

    DimensionalChange::from_changed(changed)
}

fn aggregate_resources(transaction: &HumanAiTransaction) -> BTreeMap<String, i64> {
    let mut resources = BTreeMap::new();
    for action in transaction
        .actions
        .iter()
        .filter(|action| action.status == ActionStatus::Executed)
    {
        for (key, delta) in &action.resource_delta {
            *resources.entry(key.clone()).or_insert(0) += delta;
        }
    }
    resources
}

fn effect_boundaries(transaction: &HumanAiTransaction, dimension: Dimension) -> BTreeSet<String> {
    transaction
        .actions
        .iter()
        .filter(|action| action.status == ActionStatus::Executed)
        .flat_map(|action| action.effects.iter())
        .filter(|effect| effect.dimension == dimension)
        .map(|effect| effect.boundary.clone())
        .collect()
}

fn push_issue(
    issues: &mut Vec<ProtocolIssue>,
    severity: IssueSeverity,
    code: &str,
    message: String,
) {
    issues.push(ProtocolIssue {
        severity,
        code: code.to_string(),
        message,
    });
}

fn collect_unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    kind: &str,
    issues: &mut Vec<ProtocolIssue>,
) -> BTreeSet<String> {
    let mut unique = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            issues.push(ProtocolIssue {
                severity: IssueSeverity::Error,
                code: format!("empty_{kind}_id"),
                message: format!("{kind} id must not be empty"),
            });
            continue;
        }
        if !unique.insert(id.to_string()) {
            issues.push(ProtocolIssue {
                severity: IssueSeverity::Error,
                code: format!("duplicate_{kind}_id"),
                message: format!("duplicate {kind} id {id}"),
            });
        }
    }
    unique
}

pub(crate) fn hash_json<T: Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).expect("protocol values must serialize");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

fn join_set(items: &BTreeSet<String>) -> String {
    items.iter().cloned().collect::<Vec<_>>().join(", ")
}
