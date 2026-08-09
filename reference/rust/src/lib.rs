// SPDX-License-Identifier: Apache-2.0

pub mod dimension;
pub mod live;
pub mod protocol;
pub mod security;

pub use dimension::{Dimension, DimensionalChange, Effect};
pub use live::{
    AuthorityScope, ConversationEvent, ConversationEventPayload, EventActor, EventReceipt,
    IngestError, IngressTrustPolicy, LiveConversationAssembler, LiveWitnessSnapshot,
    HTP_LIVE_VERSION,
};
pub use protocol::{
    ActionRecord, ActionStatus, AiProjection, AuthorityState, ChangeSummary, ClaimStatus,
    EvidenceKind, EvidenceRecord, FractureRecord, FractureSeverity, HumanAiTransaction, HumanNeed,
    IssueSeverity, ProtocolIssue, ProtocolState, RepairRecord, TranslationLevel, ValidationReport,
    WitnessRecord, HTP_VERSION,
};
pub use security::{
    publish_snapshot, PublicationProfile, SignatureError, SignatureVerification,
    SignedWitnessEnvelope, HTP_SIGNATURE_ALGORITHM, HTP_SIGNATURE_VERSION,
};
