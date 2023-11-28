use dlc_manager::error::Error as DaemonError;
use dlc_manager::Oracle;
use dlc_messages::oracle_msgs::{
    EventDescriptor, OracleAnnouncement, OracleAttestation, OracleEvent,
};
use lightning::util::ser::Writeable;
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

const ORACLE_ITEM_TREE: u8 = 3;

#[derive(Serialize, Deserialize)]
struct OnDiskOracleItem {
    announcement: OracleAnnouncement,
    nonces: Vec<SecretKey>,
    attestation: Option<OracleAttestation>,
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

    fn get_oracle_item(&self, event_id: &str) -> Option<OnDiskOracleItem> {
        let data = self
            .db
            .open_tree([ORACLE_ITEM_TREE])
            .unwrap()
            .get(event_id.as_bytes())
            .unwrap()?;
        Some(bincode::deserialize(&data).unwrap())
    }
}

impl Oracle for SledOracle {
    fn get_public_key(&self) -> XOnlyPublicKey {
        self.get_keypair().x_only_public_key().0
    }

    fn get_announcement(&self, event_id: &str) -> Result<OracleAnnouncement, DaemonError> {
        let item = self
            .get_oracle_item(event_id)
            .ok_or_else(|| DaemonError::OracleError("Announcement not found!".to_string()))?;

        Ok(item.announcement)
    }

    fn get_attestation(&self, event_id: &str) -> Result<OracleAttestation, DaemonError> {
        let item = self
            .get_oracle_item(event_id)
            .ok_or_else(|| DaemonError::OracleError("Attestation not found!".to_string()))?;

        if let Some(attestation) = item.attestation {
            Ok(attestation)
        } else {
            Err(DaemonError::OracleError(
                "Attestation not found!".to_string(),
            ))
        }
    }
}

impl SledOracle {
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

    pub fn add_event(&mut self, event_id: &str, event_descriptor: &EventDescriptor, maturity: u32) {
        let (priv_oracle_nonces, oracle_nonces) = self.generate_nonces_for_event(event_descriptor);
        let oracle_event = OracleEvent {
            oracle_nonces,
            event_maturity_epoch: maturity,
            event_descriptor: event_descriptor.clone(),
            event_id: event_id.to_string(),
        };
        let mut event_hex = Vec::new();
        oracle_event
            .write(&mut event_hex)
            .expect("Error writing oracle event");
        let msg = Message::from_hashed_data::<secp256k1_zkp::hashes::sha256::Hash>(&event_hex);
        let sig = self.secp.sign_schnorr(&msg, &self.get_keypair());
        let announcement = OracleAnnouncement {
            oracle_event,
            oracle_public_key: self.get_public_key(),
            announcement_signature: sig,
        };
        let item = OnDiskOracleItem {
            announcement,
            nonces: priv_oracle_nonces,
            attestation: None,
        };
        self.db
            .open_tree([ORACLE_ITEM_TREE])
            .unwrap()
            .insert(event_id.as_bytes(), bincode::serialize(&item).unwrap())
            .unwrap();
    }

    pub fn add_attestation(&mut self, event_id: &str, outcomes: &[String]) {
        let update_result = self
            .db
            .open_tree([ORACLE_ITEM_TREE])
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
                if item.attestation.is_none() {
                    item.attestation = Some(attestation);
                }
                Some(bincode::serialize(&item).unwrap())
            })
            .unwrap();

        if let Some(data) = update_result {
            self.db
                .open_tree([ORACLE_ITEM_TREE])
                .unwrap()
                .insert(event_id.as_bytes(), data)
                .unwrap();
        } else {
            panic!("Failed to update attestation! Annoucement not found!");
        }
    }
}
