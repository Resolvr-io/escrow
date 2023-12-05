use core::panic;
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{OracleAnnouncement, OracleAttestation};
use escrow_agent_messages::{AdjudicationRequest, AdjudicationRequestStatus, EscrowAgent};

pub const BOUNTY_COMPLETE_ORACLE_MESSAGE: &str = "BOUNTY_COMPLETE";
pub const BOUNTY_INSUFFICIENT_ORACLE_MESSAGE: &str = "BOUNTY_INSUFFICIENT";

pub struct NostrNip4ResolvrOracle {}

impl NostrNip4ResolvrOracle {
    // TODO: Add an nPub key to the constructor (I don't know what type it is yet).
    pub fn new_from_npub() -> Self {
        Self {}
    }
}

impl EscrowAgent for NostrNip4ResolvrOracle {
    fn request_adjudication(
        &self,
        _adjudication_request: AdjudicationRequest,
    ) -> Result<AdjudicationRequestStatus, String> {
        // TODO: Implement this.
        panic!("Not implemented yet.");
    }

    fn get_adjudication_request_status(
        &self,
        _event_id: &str,
    ) -> Result<AdjudicationRequestStatus, String> {
        // TODO: Implement this.
        panic!("Not implemented yet.");
    }
}

impl Oracle for NostrNip4ResolvrOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        // TODO: Implement this.
        panic!("Not implemented yet.");
    }

    fn get_announcement(
        &self,
        _event_id: &str,
    ) -> Result<OracleAnnouncement, dlc_manager::error::Error> {
        // TODO: Implement this.
        panic!("Not implemented yet.");
    }

    fn get_attestation(
        &self,
        _event_id: &str,
    ) -> Result<OracleAttestation, dlc_manager::error::Error> {
        // TODO: Implement this.
        panic!("Not implemented yet.");
    }
}
