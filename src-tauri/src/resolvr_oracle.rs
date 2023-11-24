use bitcoin::secp256k1::rand::{thread_rng, RngCore};
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{
    EnumEventDescriptor, EventDescriptor, OracleAnnouncement, OracleAttestation,
};
use escrow_agent_messages::{AdjudicationRequest, AdjudicationRequestStatus, EscrowAgent};
use mocks::mock_oracle_provider::MockOracle;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub const BOUNTY_COMPLETE_ORACLE_MESSAGE: &str = "BOUNTY_COMPLETE";
pub const BOUNTY_INSUFFICIENT_ORACLE_MESSAGE: &str = "BOUNTY_INSUFFICIENT";

pub enum BountyOutcome {
    Complete,
    Insufficient,
}

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
        panic!("Not implemented yet.");
    }

    fn get_adjudication_request_status(
        &self,
        _event_id: &str,
    ) -> Result<AdjudicationRequestStatus, String> {
        panic!("Not implemented yet.");
    }
}

impl ToString for BountyOutcome {
    fn to_string(&self) -> String {
        match self {
            BountyOutcome::Complete => BOUNTY_COMPLETE_ORACLE_MESSAGE.to_string(),
            BountyOutcome::Insufficient => BOUNTY_INSUFFICIENT_ORACLE_MESSAGE.to_string(),
        }
    }
}

pub struct ResolvrOracle {
    mock_oracle: Arc<Mutex<MockOracle>>,
}

impl ResolvrOracle {
    pub fn new_from_generated_keypair() -> ResolvrOracle {
        ResolvrOracle {
            mock_oracle: Arc::from(Mutex::from(MockOracle::new())),
        }
    }

    /// Creates a new announcement and returns the event ID.
    pub fn create_announcement(&self) -> String {
        let event_id = Self::generate_new_event_id();

        let event_descriptor = EventDescriptor::EnumEvent(EnumEventDescriptor {
            outcomes: vec![
                BOUNTY_COMPLETE_ORACLE_MESSAGE.to_string(),
                BOUNTY_INSUFFICIENT_ORACLE_MESSAGE.to_string(),
            ],
        });

        let maturity_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            + 10;

        self.mock_oracle
            .lock()
            .unwrap()
            .add_event(&event_id, &event_descriptor, maturity_epoch);
        event_id
    }

    /// Creates an attestation for the given event ID, or returns an error if the event ID is not
    /// found or the event has already been attested.
    pub fn create_attestation(
        &self,
        event_id: &str,
        outcome: BountyOutcome,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mock_oracle
            .lock()
            .unwrap()
            .add_attestation(event_id, &[outcome.to_string()]);
        Ok(())
    }

    /// Creates an new unique hex-encoded 256-bit event ID.
    fn generate_new_event_id() -> String {
        let mut rng = thread_rng();
        let mut event_id = [0u8; 32];
        rng.fill_bytes(&mut event_id);
        hex::encode(event_id)
    }
}

impl Oracle for ResolvrOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        self.mock_oracle.lock().unwrap().get_public_key()
    }

    fn get_announcement(
        &self,
        event_id: &str,
    ) -> Result<OracleAnnouncement, dlc_manager::error::Error> {
        self.mock_oracle.lock().unwrap().get_announcement(event_id)
    }

    fn get_attestation(
        &self,
        event_id: &str,
    ) -> Result<OracleAttestation, dlc_manager::error::Error> {
        self.mock_oracle.lock().unwrap().get_attestation(event_id)
    }
}
