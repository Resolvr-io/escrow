use bitcoin::secp256k1::{
    rand::{thread_rng, RngCore},
    Message,
};
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{
    EnumEventDescriptor, EventDescriptor, OracleAnnouncement, OracleAttestation, OracleEvent,
};
use lightning::util::ser::Writeable;
use std::{collections::HashMap, sync::Mutex, time::SystemTime};

pub const BOUNTY_COMPLETE_ORACLE_MESSAGE: &str = "BOUNTY_COMPLETE";
pub const BOUNTY_INSUFFICIENT_ORACLE_MESSAGE: &str = "BOUNTY_INSUFFICIENT";

pub enum BountyOutcome {
    Complete,
    Insufficient,
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
    /// The private key of the oracle.
    private_key: bitcoin::secp256k1::SecretKey,

    /// The public key of the oracle.
    public_key: bitcoin::secp256k1::PublicKey,

    /// Map of event ID to announcement and attestation (if it has been made).
    events: Mutex<HashMap<String, (OracleAnnouncement, Option<OracleAttestation>)>>,
}

impl ResolvrOracle {
    pub fn new_from_generated_keypair() -> ResolvrOracle {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut thread_rng());
        ResolvrOracle {
            private_key,
            public_key,
            events: Mutex::from(HashMap::new()),
        }
    }

    /// Creates a new announcement and returns the event ID.
    pub fn create_announcement(&self) -> String {
        let event_id = Self::generate_new_event_id();

        let oracle_event = OracleEvent {
            oracle_nonces: vec![Self::generate_nonce()],
            event_maturity_epoch: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
                + 10,
            event_descriptor: EventDescriptor::EnumEvent(EnumEventDescriptor {
                outcomes: vec![
                    BOUNTY_COMPLETE_ORACLE_MESSAGE.to_string(),
                    BOUNTY_INSUFFICIENT_ORACLE_MESSAGE.to_string(),
                ],
            }),
            event_id: event_id.clone(),
        };

        let announcement_signature = {
            let mut event_hex = Vec::new();
            oracle_event
                .write(&mut event_hex)
                .expect("Error writing oracle event");

            let msg =
                Message::from_hashed_data::<bitcoin::secp256k1::hashes::sha256::Hash>(&event_hex);

            let secp = bitcoin::secp256k1::Secp256k1::new();
            secp.sign_schnorr(&msg, &self.private_key.keypair(&secp))
        };

        let announcement = OracleAnnouncement {
            announcement_signature,
            oracle_public_key: self.get_public_key(),
            oracle_event,
        };

        self.events
            .lock()
            .unwrap()
            .insert(event_id.clone(), (announcement, None));
        event_id
    }

    /// Creates an attestation for the given event ID, or returns an error if the event ID is not
    /// found or the event has already been attested.
    pub fn create_attestation(
        &self,
        event_id: &str,
        outcome: BountyOutcome,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (announcement, attestation) = self
            .events
            .lock()
            .unwrap()
            .get(event_id)
            .ok_or("Event ID not found.")?
            .clone();

        if attestation.is_some() {
            return Err("Event has already been attested.".into());
        }

        // TODO - Ensure this is the correct way to produce the event signature.
        let outcome_signature = {
            let mut event_hex = Vec::new();
            outcome
                .to_string()
                .write(&mut event_hex)
                .expect("Error writing oracle event");

            let msg =
                Message::from_hashed_data::<bitcoin::secp256k1::hashes::sha256::Hash>(&event_hex);

            let secp = bitcoin::secp256k1::Secp256k1::new();
            secp.sign_schnorr(&msg, &self.private_key.keypair(&secp))
        };

        let attestation = OracleAttestation {
            oracle_public_key: self.get_public_key(),
            signatures: vec![outcome_signature],
            outcomes: vec![outcome.to_string()],
        };

        self.events
            .lock()
            .unwrap()
            .insert(event_id.to_string(), (announcement, Some(attestation)));

        Ok(())
    }

    /// Creates an new unique hex-encoded 256-bit event ID.
    fn generate_new_event_id() -> String {
        let mut rng = thread_rng();
        let mut event_id = [0u8; 32];
        rng.fill_bytes(&mut event_id);
        hex::encode(event_id)
    }

    /// Generates a new unique nonce.
    fn generate_nonce() -> bitcoin::XOnlyPublicKey {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let (_private_key, public_key) = secp.generate_keypair(&mut thread_rng());
        public_key.x_only_public_key().0
    }
}

impl Oracle for ResolvrOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        self.public_key.x_only_public_key().0
    }

    fn get_announcement(
        &self,
        event_id: &str,
    ) -> Result<OracleAnnouncement, dlc_manager::error::Error> {
        return match self.events.lock().unwrap().get(event_id) {
            Some((announcement, _)) => Ok(announcement.clone()),
            None => Err(dlc_manager::error::Error::OracleError(
                "Event ID not found.".to_string(),
            )),
        };
    }

    fn get_attestation(
        &self,
        event_id: &str,
    ) -> Result<OracleAttestation, dlc_manager::error::Error> {
        return match self.events.lock().unwrap().get(event_id) {
            Some((_, Some(attestation))) => Ok(attestation.clone()),
            Some((_, None)) => Err(dlc_manager::error::Error::OracleError(
                "Attestation not found.".to_string(),
            )),
            None => Err(dlc_manager::error::Error::OracleError(
                "Event ID not found.".to_string(),
            )),
        };
    }
}
