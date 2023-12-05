use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdjudicationRequest {
    pub bounty_template: BountyTemplate,
}

#[derive(Serialize, Deserialize)]
pub struct AdjudicationRequestStatus {
    /// The event ID of the bounty. ID is not usable until the bounty is
    /// approved (which can be checked with `adjudication_state`).
    pub oracle_event_id: String,

    pub adjudication_state: AdjudicationRequestState,
}

#[derive(Serialize, Deserialize)]
pub struct BountyTemplate {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub enum AdjudicationRequestState {
    Approved,
    Denied,
    InReview,
}
