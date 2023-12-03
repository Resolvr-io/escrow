use core::panic;
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{OracleAnnouncement, OracleAttestation};
use serde::{Deserialize, Serialize};

pub const BOUNTY_COMPLETE_ORACLE_MESSAGE: &str = "BOUNTY_COMPLETE";
pub const BOUNTY_INSUFFICIENT_ORACLE_MESSAGE: &str = "BOUNTY_INSUFFICIENT";

pub struct NostrNip4ResolvrOracle {}

impl NostrNip4ResolvrOracle {
    // TODO: Add an nPub key to the constructor (I don't know what type it is yet).
    pub fn new_from_npub() -> Self {
        Self {}
    }

    pub async fn request_adjudication(
        &self,
        _adjudication_request: AdjudicationRequest,
    ) -> Result<AdjudicationRequestStatus, String> {
        panic!("Not implemented yet.");
    }

    pub async fn get_adjudication_request_status(
        &self,
        _event_id: &str,
    ) -> Result<AdjudicationRequestStatus, String> {
        panic!("Not implemented yet.");
    }
}

impl Oracle for NostrNip4ResolvrOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        panic!("Not implemented yet.");
    }

    fn get_announcement(
        &self,
        _event_id: &str,
    ) -> Result<OracleAnnouncement, dlc_manager::error::Error> {
        panic!("Not implemented yet.");
    }

    fn get_attestation(
        &self,
        _event_id: &str,
    ) -> Result<OracleAttestation, dlc_manager::error::Error> {
        panic!("Not implemented yet.");
    }
}

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
}

#[derive(Serialize, Deserialize)]
pub enum AdjudicationRequestState {
    Approved,
    Denied,
    InReview,
}
