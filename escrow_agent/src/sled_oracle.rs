use dlc_manager::error::Error as DaemonError;
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{EventDescriptor, OracleAnnouncement, OracleAttestation};
use escrow_agent_messages::{
    AdjudicationRequest, AdjudicationRequestState, AdjudicationRequestStatus, BountyTemplate,
    EscrowAgent,
};
use secp256k1_zkp::rand::thread_rng;
use secp256k1_zkp::serde::{Deserialize, Serialize};
use secp256k1_zkp::{All, Message, Secp256k1, SecretKey};
use secp256k1_zkp::{KeyPair, XOnlyPublicKey};
use sled::Db;

const DB_VERSION_TREE: u8 = 1;
const DB_VERSION_KEY: u8 = 1;
const DB_LATEST_VERSION: u8 = 1;

const KEYPAIR_TREE: u8 = 2;
const KEYPAIR_KEY: u8 = 1;

const ADJUDICATION_ITEM_TREE: u8 = 3;

#[derive(Serialize, Deserialize)]
struct OnDiskAdjudicationItem {
    bounty_template: BountyTemplate,
    adjudication_state: OnDiskAdjudicationItemState,
}

impl From<OnDiskAdjudicationItem> for AdjudicationRequestStatus {
    fn from(item: OnDiskAdjudicationItem) -> Self {
        AdjudicationRequestStatus {
            oracle_event_id: item.bounty_template.oracle_event_id,
            adjudication_state: match item.adjudication_state {
                OnDiskAdjudicationItemState::Approved(_) => AdjudicationRequestState::Approved,
                OnDiskAdjudicationItemState::Denied(_) => AdjudicationRequestState::Denied,
                OnDiskAdjudicationItemState::InReview(_) => AdjudicationRequestState::InReview,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
enum OnDiskAdjudicationItemState {
    Approved(Box<OnDiskOracleItem>),
    Denied(EventId),
    InReview(EventId),
}

type EventId = String;

#[derive(Serialize, Deserialize)]
struct OnDiskOracleItem {
    announcement: OracleAnnouncement,
    nonces: Vec<SecretKey>,
    attestation_or: Option<OracleAttestation>,
}

#[derive(Clone, Debug)]
pub struct SledOracle {
    db: Db,
    secp: Secp256k1<All>,
}

impl SledOracle {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let oracle = SledOracle {
            db: sled::open(path)?,
            secp: Secp256k1::new(),
        };
        oracle.maybe_insert_db_version();
        oracle.migrate_db();
        oracle.assert_db_version();
        oracle.maybe_insert_new_keypair();
        println!("Oracle public key: {:x}", oracle.get_public_key());
        oracle.db.flush().unwrap();
        Ok(oracle)
    }

    /// Returns a list of all adjudication requests in the given state, or all requests if `state_or` is `None`.
    pub fn list_adjudication_requests(
        &self,
        state_or: Option<AdjudicationRequestState>,
    ) -> Vec<AdjudicationRequestStatus> {
        let mut result = Vec::new();
        let tree = self.db.open_tree([ADJUDICATION_ITEM_TREE]).unwrap();
        for item in tree.iter() {
            let (_, data) = item.unwrap();
            let item: OnDiskAdjudicationItem = bincode::deserialize(&data).unwrap();
            let status: AdjudicationRequestStatus = item.into();
            match &state_or {
                Some(state) => {
                    if status.adjudication_state == *state {
                        result.push(status);
                    }
                }
                None => result.push(status),
            };
        }
        result
    }

    pub fn approve_adjudication_request(&mut self, _event_id: &str) -> Result<(), DaemonError> {
        // TODO: Implement this.
        panic!("Not implemented!");

        // let event_descriptor: &EventDescriptor;
        // let maturity: u32;
        // let (nonces, oracle_nonces) = self.generate_nonces_for_event(event_descriptor);
        // let oracle_event = OracleEvent {
        //     oracle_nonces,
        //     event_maturity_epoch: maturity,
        //     event_descriptor: event_descriptor.clone(),
        //     event_id: event_id.to_string(),
        // };
        // let mut event_hex = Vec::new();
        // oracle_event
        //     .write(&mut event_hex)
        //     .expect("Error writing oracle event");
        // let msg = Message::from_hashed_data::<secp256k1_zkp::hashes::sha256::Hash>(&event_hex);
        // let sig = self.secp.sign_schnorr(&msg, &self.get_keypair());
        // let announcement = OracleAnnouncement {
        //     announcement_signature: sig,
        //     oracle_public_key: self.get_public_key(),
        //     oracle_event,
        // };
        // let oracle_item = OnDiskOracleItem {
        //     announcement,
        //     nonces,
        //     attestation_or: None,
        // };
    }

    pub fn deny_adjudication_request(&mut self, event_id: &str) -> Result<(), DaemonError> {
        // TODO: Implement this.
        panic!("Not implemented!");
    }

    pub fn attest_adjudication(&mut self, event_id: &str, outcomes: &[String]) {
        let update_result = self
            .db
            .open_tree([ADJUDICATION_ITEM_TREE])
            .unwrap()
            .update_and_fetch(event_id.as_bytes(), |old: Option<&[u8]>| {
                let mut item: OnDiskOracleItem = match old {
                    Some(data) => bincode::deserialize(data).unwrap(),
                    None => return None,
                };
                let signatures = outcomes
                    .iter()
                    .zip(item.nonces.iter())
                    .map(|(x, nonce)| {
                        let msg = Message::from_hashed_data::<secp256k1_zkp::hashes::sha256::Hash>(
                            x.as_bytes(),
                        );
                        dlc::secp_utils::schnorrsig_sign_with_nonce(
                            &self.secp,
                            &msg,
                            &self.get_keypair(),
                            nonce.as_ref(),
                        )
                    })
                    .collect();
                let attestation = OracleAttestation {
                    oracle_public_key: self.get_public_key(),
                    signatures,
                    outcomes: outcomes.to_vec(),
                };
                // IMPORTANT: Only update the attestation if it is not already set.
                // This is to prevent the oracle from overwriting a previous attestation, which
                // could cause the oracle to produce multiple attestations for the same event.
                // If multiple attestations are produced, the oracle's private key can be computed
                // by anyone who knows the public key and the two attestations.
                if item.attestation_or.is_none() {
                    item.attestation_or = Some(attestation);
                }
                Some(bincode::serialize(&item).unwrap())
            })
            .unwrap();

        if let Some(data) = update_result {
            self.db
                .open_tree([ADJUDICATION_ITEM_TREE])
                .unwrap()
                .insert(event_id.as_bytes(), data)
                .unwrap();
        } else {
            panic!("Failed to update attestation! Announcement not found!");
        }
    }

    fn generate_nonces_for_event(
        &mut self,
        event_descriptor: &EventDescriptor,
    ) -> (Vec<SecretKey>, Vec<XOnlyPublicKey>) {
        let nb_nonces = match event_descriptor {
            EventDescriptor::EnumEvent(_) => 1,
            EventDescriptor::DigitDecompositionEvent(d) => d.nb_digits,
        };

        let priv_nonces: Vec<_> = (0..nb_nonces)
            .map(|_| SecretKey::new(&mut thread_rng()))
            .collect();
        let key_pairs: Vec<_> = priv_nonces
            .iter()
            .map(|x| KeyPair::from_seckey_slice(&self.secp, x.as_ref()).unwrap())
            .collect();

        let nonces = key_pairs
            .iter()
            .map(|k| XOnlyPublicKey::from_keypair(k).0)
            .collect();

        (priv_nonces, nonces)
    }

    fn maybe_insert_db_version(&self) {
        let result = self
            .db
            .open_tree([DB_VERSION_TREE])
            .unwrap()
            .compare_and_swap(
                [DB_VERSION_KEY],
                None as Option<&[u8]>,
                Some(&[DB_LATEST_VERSION]),
            )
            .unwrap();

        if let Err(data) = result {
            // Check that the version is correct.
            assert_eq!(data.current.unwrap(), [DB_LATEST_VERSION]);
        }
    }

    fn migrate_db(&self) {
        // TODO: Implement this.
    }

    fn assert_db_version(&self) {
        let version = self
            .db
            .open_tree([DB_VERSION_TREE])
            .unwrap()
            .get([DB_VERSION_KEY])
            .unwrap()
            .expect("DB version not found!");
        // TODO: This should be a proper error rather than a panic.
        assert_eq!(version, [DB_LATEST_VERSION]);
    }

    /// Initialize the db with a new keypair if it is empty. Otherwise, do nothing.
    /// Returns true if a new keypair was inserted.
    fn maybe_insert_new_keypair(&self) -> bool {
        let new_key_pair = KeyPair::new(&self.secp, &mut thread_rng());

        self.db
            .open_tree([KEYPAIR_TREE])
            .unwrap()
            .compare_and_swap(
                [KEYPAIR_KEY],
                None as Option<&[u8]>,
                Some(bincode::serialize(&new_key_pair).unwrap()),
            )
            .unwrap()
            .is_ok()
    }

    fn get_keypair(&self) -> KeyPair {
        let data = self
            .db
            .open_tree([KEYPAIR_TREE])
            .unwrap()
            .get([KEYPAIR_KEY])
            .unwrap()
            .expect("Key pair not found!");
        bincode::deserialize(&data).unwrap()
    }

    fn get_adjudication_item(&self, event_id: &str) -> Option<OnDiskAdjudicationItem> {
        let data = self
            .db
            .open_tree([ADJUDICATION_ITEM_TREE])
            .unwrap()
            .get(event_id.as_bytes())
            .unwrap()?;
        Some(bincode::deserialize(&data).unwrap())
    }
}

impl EscrowAgent for SledOracle {
    fn request_adjudication(
        &self,
        adjudication_request: AdjudicationRequest,
    ) -> Result<AdjudicationRequestStatus, String> {
        let oracle_event_id = adjudication_request.bounty_template.oracle_event_id.clone();
        let item = OnDiskAdjudicationItem {
            bounty_template: adjudication_request.bounty_template,
            adjudication_state: OnDiskAdjudicationItemState::InReview(oracle_event_id.to_string()),
        };
        match self
            .db
            .open_tree([ADJUDICATION_ITEM_TREE])
            .unwrap()
            .compare_and_swap(
                oracle_event_id.as_bytes(),
                None as Option<&[u8]>,
                Some(bincode::serialize(&item).unwrap()),
            )
            .unwrap()
        {
            Ok(_) => Ok(item.into()),
            Err(_) => {
                Err("Adjudication request/item already exists for this event id!".to_string())
            }
        }
    }

    fn get_adjudication_request_status(
        &self,
        event_id: &str,
    ) -> Result<AdjudicationRequestStatus, String> {
        self.get_adjudication_item(event_id)
            .map(|item| item.into())
            .ok_or_else(|| "Adjudication request/item not found!".to_string())
    }
}

impl Oracle for SledOracle {
    fn get_public_key(&self) -> XOnlyPublicKey {
        self.get_keypair().x_only_public_key().0
    }

    fn get_announcement(&self, event_id: &str) -> Result<OracleAnnouncement, DaemonError> {
        let item = self
            .get_adjudication_item(event_id)
            .ok_or_else(|| DaemonError::OracleError("Announcement not found!".to_string()))?;

        match item.adjudication_state {
            OnDiskAdjudicationItemState::Approved(item) => Ok(item.announcement),
            _ => Err(DaemonError::OracleError(
                "Announcement not found!".to_string(),
            )),
        }
    }

    fn get_attestation(&self, event_id: &str) -> Result<OracleAttestation, DaemonError> {
        let item = self
            .get_adjudication_item(event_id)
            .ok_or_else(|| DaemonError::OracleError("Attestation not found!".to_string()))?;

        let attestation_or: Option<OracleAttestation> = match item.adjudication_state {
            OnDiskAdjudicationItemState::Approved(item) => item.attestation_or,
            _ => {
                return Err(DaemonError::OracleError(
                    "Attestation not found!".to_string(),
                ))
            }
        };

        if let Some(attestation) = attestation_or {
            Ok(attestation)
        } else {
            Err(DaemonError::OracleError(
                "Attestation not found!".to_string(),
            ))
        }
    }
}
