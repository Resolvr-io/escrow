use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdjudicationRequest {
    bounty_template: BountyTemplate,
}

#[derive(Serialize, Deserialize)]
pub struct AdjudicationRequestStatus {
    /// The event ID of the bounty. ID is not usable until the bounty is
    /// approved (which can be checked with `adjudication_state`).
    oracle_event_id: String,

    adjudication_state: AdjudicationRequestState,
}

#[derive(Serialize, Deserialize)]
pub struct BountyTemplate {
    title: String,
    description: String,
    oracle_event_id: String,
}

#[derive(Serialize, Deserialize)]
enum AdjudicationRequestState {
    Approved,
    Denied,
    InReview,
}
