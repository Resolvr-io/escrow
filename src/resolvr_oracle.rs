use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{OracleAnnouncement, OracleAttestation};

pub struct ResolvrOracle {}

impl Oracle for ResolvrOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_announcement(
        &self,
        event_id: &str,
    ) -> Result<OracleAnnouncement, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_attestation(
        &self,
        event_id: &str,
    ) -> Result<OracleAttestation, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }
}
