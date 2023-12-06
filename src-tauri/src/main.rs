// Prevents additional console window on Windows in release. DO NOT REMOVE!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod contract;
mod resolvr_oracle;

use bitcoin::secp256k1::PublicKey;
use bitcoin::XOnlyPublicKey;
use bitcoin_rpc_provider::BitcoinCoreProvider;
use contract::JsonContract;
use dlc::EnumerationPayout;
use dlc_manager::contract::contract_input::{ContractInput, ContractInputInfo, OracleInput};
use dlc_manager::contract::enum_descriptor::EnumDescriptor;
use dlc_manager::contract::{Contract, ContractDescriptor};
use dlc_manager::Oracle;
use dlc_manager::Storage;
use dlc_manager::SystemTimeProvider;
use dlc_sled_storage_provider::SledStorageProvider;
use escrow_agent_messages::EscrowAgent;
use escrow_agent_messages::{AdjudicationRequest, AdjudicationRequestStatus};
use resolvr_oracle::{
    NostrNip4ResolvrOracle, BOUNTY_COMPLETE_ORACLE_MESSAGE, BOUNTY_INSUFFICIENT_ORACLE_MESSAGE,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};

use keyring::Entry;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn save_secret_key_to_keychain(nsec: &str, npub: &str) -> String {
    let entry = match Entry::new("resolvr", npub) {
        Ok(entry) => entry,
        Err(_e) => return "error".to_string(),
    };

    if let Err(_e) = entry.set_password(nsec) {
        return "error".to_string();
    }

    "success".to_string()
}

#[tauri::command]
fn get_nsec(npub: &str) -> String {
    let entry = match Entry::new("resolvr", npub) {
        Ok(entry) => entry,
        Err(_e) => return "error".to_string(),
    };

    match entry.get_password() {
        Ok(password) => password.to_string(),
        Err(_e) => "error".to_string(),
    }
}

#[tauri::command]
fn request_oracle_adjudication(
    adjudication_request: AdjudicationRequest,
    oracle: tauri::State<Arc<NostrNip4ResolvrOracle>>,
) -> Result<AdjudicationRequestStatus, String> {
    oracle.request_adjudication(adjudication_request)
}

#[tauri::command]
fn get_oracle_adjudication_request_status(
    oracle_event_id: &str,
    oracle: tauri::State<Arc<NostrNip4ResolvrOracle>>,
) -> Result<AdjudicationRequestStatus, String> {
    oracle.get_adjudication_request_status(oracle_event_id)
}

#[tauri::command]
fn connect_to_bitcoin_core(
    bitcoin_core_config: BitcoinCoreConfig,
    oracle: tauri::State<'_, Arc<NostrNip4ResolvrOracle>>,
    dlc_storage: tauri::State<Arc<SledStorageProvider>>,
    dlc_manager_or: tauri::State<Arc<Mutex<Option<ResolvrDlcManager>>>>,
) -> Result<(), String> {
    let bitcoin_core_provider = match BitcoinCoreProvider::new(
        bitcoin_core_config.host.clone(),
        bitcoin_core_config.port,
        None,
        bitcoin_core_config.rpc_user.clone(),
        bitcoin_core_config.rpc_password.clone(),
    ) {
        Ok(v) => Arc::new(v),
        Err(e) => return Err(format!("Error creating Bitcoin Core provider: {}", e)),
    };

    let mut oracles: HashMap<XOnlyPublicKey, Arc<NostrNip4ResolvrOracle>> = HashMap::new();
    oracles.insert(oracle.get_public_key(), oracle.inner().clone());

    let mut dlc_manager_or: MutexGuard<Option<ResolvrDlcManager>> = dlc_manager_or.lock().unwrap();
    let binding: &mut Option<ResolvrDlcManager> = dlc_manager_or.deref_mut();
    match binding {
        Some(_) => return Err(String::from("DLC manager already initialized.")),
        None => {
            let dlc_manager = match dlc_manager::manager::Manager::new(
                bitcoin_core_provider.clone(),
                bitcoin_core_provider.clone(),
                dlc_storage.inner().clone(),
                oracles,
                Arc::new(dlc_manager::SystemTimeProvider {}),
                bitcoin_core_provider,
            ) {
                Ok(v) => v,
                Err(e) => return Err(format!("Error creating DLC manager: {}", e)),
            };

            *binding = Some(dlc_manager);
        }
    };

    Ok(())
}

#[tauri::command]
fn get_contracts(
    dlc_storage: tauri::State<Arc<SledStorageProvider>>,
) -> Result<Vec<JsonContract>, String> {
    let contracts: Vec<Contract> = match dlc_storage.get_contracts() {
        Ok(v) => v,
        Err(e) => return Err(format!("Error getting contracts: {}", e)),
    };

    Ok(contracts.iter().map(JsonContract::from).collect())
}

#[tauri::command]
fn offer_contract(
    bounty_amount_sats: u64,
    taker_collateral_sats: u64,
    fee_rate_sats_per_vbyte: u64,
    oracle_event_id: String,
    counter_party_public_key: &str,
    dlc_manager_or: tauri::State<Arc<Mutex<Option<ResolvrDlcManager>>>>,
    oracle: tauri::State<Arc<NostrNip4ResolvrOracle>>,
) -> Result<(), String> {
    let mut dlc_manager_or = dlc_manager_or.lock().unwrap();
    let mut binding = dlc_manager_or.as_mut();
    let dlc_manager = match &mut binding {
        Some(m) => m,
        None => return Err(String::from("DLC manager not initialized.")),
    };

    let public_key = match bitcoin::secp256k1::PublicKey::from_str(counter_party_public_key) {
        Ok(pk) => pk,
        Err(e) => return Err(format!("Error parsing public key: {}", e)),
    };

    let dlc_contract = create_bounty_contract(
        bounty_amount_sats,
        taker_collateral_sats,
        fee_rate_sats_per_vbyte,
        oracle.get_public_key(),
        oracle_event_id,
    );

    match dlc_manager.send_offer(&dlc_contract, public_key) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error sending contract offer: {}", e)),
    }
}

#[tauri::command]
fn accept_contract(
    contract_id: String,
    dlc_manager_or: tauri::State<Arc<Mutex<Option<ResolvrDlcManager>>>>,
    dlc_msg_handler: tauri::State<Arc<NostrNip4DlcMessageHandler>>,
) -> Result<JsonContract, String> {
    let mut dlc_manager_or = dlc_manager_or.lock().unwrap();
    let mut binding = dlc_manager_or.as_mut();
    let dlc_manager = match &mut binding {
        Some(m) => m,
        None => return Err(String::from("DLC manager not initialized.")),
    };

    let contract_id_bytes = match hex::decode(contract_id) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Error decoding contract ID: {}. ID must be a hex string.",
                e
            ))
        }
    };

    let contract_id: [u8; 32] = match contract_id_bytes.try_into() {
        Ok(v) => v,
        Err(_) => {
            return Err(String::from(
                "Error decoding contract ID. ID must be a 32-byte hex string.",
            ))
        }
    };

    let (_, counter_party, accept_dlc) = match dlc_manager.accept_contract_offer(&contract_id) {
        Ok(res) => res,
        Err(e) => return Err(format!("Error accepting contract: {}", e)),
    };
    dlc_msg_handler.send_msg(dlc_messages::Message::Accept(accept_dlc), counter_party);

    let contract = match dlc_manager.get_store().get_contract(&contract_id) {
        Ok(v) => match v {
            Some(v) => v,
            None => return Err(String::from("Contract not found.")),
        },
        Err(e) => return Err(format!("Error getting contract: {}", e)),
    };

    Ok((&contract).into())
}

#[tauri::command]
fn delete_contract(
    contract_id: String,
    dlc_manager_or: tauri::State<Arc<Mutex<Option<ResolvrDlcManager>>>>,
) -> Result<(), String> {
    let mut dlc_manager_or = dlc_manager_or.lock().unwrap();
    let mut binding = dlc_manager_or.as_mut();
    let dlc_manager = match &mut binding {
        Some(m) => m,
        None => return Err(String::from("DLC manager not initialized.")),
    };

    let contract_id_bytes = match hex::decode(contract_id) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Error decoding contract ID: {}. ID must be a hex string.",
                e
            ))
        }
    };

    let contract_id: [u8; 32] = match contract_id_bytes.try_into() {
        Ok(v) => v,
        Err(_) => {
            return Err(String::from(
                "Error decoding contract ID. ID must be a 32-byte hex string.",
            ))
        }
    };

    match dlc_manager.get_store().delete_contract(&contract_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error deleting contract: {}", e)),
    }
}

struct NostrNip4DlcMessageHandler {}

impl NostrNip4DlcMessageHandler {
    fn new() -> Self {
        Self {}
    }

    /// Sends a message to the given counterparty to progress the state of a DLC
    /// contract.
    fn send_msg(&self, _msg: dlc_messages::Message, _counter_party: PublicKey) {
        panic!("Not implemented.");
    }

    /// Returns the next incoming messages and the counterparty public key that
    /// sent it, without removing it from the queue.
    fn peek_next_incoming_msg(&self) -> &Option<(dlc_messages::Message, PublicKey)> {
        panic!("Not implemented.");
    }

    /// Returns the next incoming messages and the counterparty public key that
    /// sent it, removing it from the queue.
    fn pop_next_incoming_msg(&self) -> Option<(dlc_messages::Message, PublicKey)> {
        panic!("Not implemented.");
    }
}

fn process_incoming_dlc_msgs(
    dlc_manager: &mut ResolvrDlcManager,
    dlc_msg_handler: &NostrNip4DlcMessageHandler,
) -> Result<(), String> {
    loop {
        let (msg, counter_party) = match dlc_msg_handler.peek_next_incoming_msg() {
            Some(next_msg) => next_msg,
            // If there are no more messages, stop processing.
            None => return Ok(()),
        };

        match dlc_manager.on_dlc_message(msg, *counter_party) {
            Ok(_) => {
                // Remove the message from the queue since it was processed
                // successfully.
                dlc_msg_handler.pop_next_incoming_msg();
            }
            Err(e) => return Err(format!("Error processing message: {}", e)),
        };
    }
}

#[derive(Serialize, Deserialize)]
struct BitcoinCoreConfig {
    host: String,
    port: u16,
    rpc_user: String,
    rpc_password: String,
}

type ResolvrDlcManager = dlc_manager::manager::Manager<
    Arc<BitcoinCoreProvider>,
    Arc<BitcoinCoreProvider>,
    Arc<SledStorageProvider>,
    Arc<NostrNip4ResolvrOracle>,
    Arc<SystemTimeProvider>,
    Arc<BitcoinCoreProvider>,
>;

#[tokio::main]
async fn main() {
    let context = tauri::generate_context!();
    let app_local_data_dir = tauri::api::path::app_local_data_dir(context.config())
        .expect("Error getting app local data dir.");

    // TODO: Set the nPub to our hosted oracle (once it exists).
    let oracle = Arc::from(NostrNip4ResolvrOracle::new_from_npub());

    let dlc_msg_handler = Arc::from(NostrNip4DlcMessageHandler::new());

    let dlc_storage_provider: Arc<SledStorageProvider> = Arc::new(
        SledStorageProvider::new(&format!(
            "{}/dlc_db_hackathon",
            app_local_data_dir
                .to_str()
                .expect("Error converting app local data dir to string.")
        ))
        .expect("Error creating DLC storage."),
    );

    let dlc_manager_or: Arc<Mutex<Option<ResolvrDlcManager>>> = Arc::new(Mutex::new(None));

    let dlc_msg_handler_clone = dlc_msg_handler.clone();
    let dlc_manager_or_clone = dlc_manager_or.clone();
    tokio::task::spawn(async move {
        loop {
            if let Some(dlc_manager) = dlc_manager_or_clone.lock().unwrap().as_mut() {
                if let Err(e) = process_incoming_dlc_msgs(dlc_manager, &dlc_msg_handler_clone) {
                    // TODO: Handle error.
                    println!("Error processing incoming DLC messages: {}", e);
                };
            };
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_secret_key_to_keychain,
            get_nsec,
            request_oracle_adjudication,
            get_oracle_adjudication_request_status,
            connect_to_bitcoin_core,
            get_contracts,
            offer_contract,
            accept_contract,
            delete_contract
        ])
        .manage(oracle)
        .manage(dlc_msg_handler)
        .manage(dlc_storage_provider)
        .manage(dlc_manager_or)
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(context)
        .expect("Error while running Tauri application.");
}

/// Create a DLC contract template for a bounty.
fn create_bounty_contract(
    bounty_amount_sats: u64,
    taker_collateral_sats: u64,
    fee_rate_sats_per_vbyte: u64,
    oracle_public_key: XOnlyPublicKey,
    oracle_event_id: String,
) -> ContractInput {
    ContractInput {
        offer_collateral: bounty_amount_sats,
        accept_collateral: taker_collateral_sats,
        fee_rate: fee_rate_sats_per_vbyte,
        contract_infos: vec![ContractInputInfo {
            contract_descriptor: ContractDescriptor::Enum(EnumDescriptor {
                outcome_payouts: vec![
                    // If the bounty is completed, the taker gets the bounty
                    // amount plus their collateral back.
                    EnumerationPayout {
                        outcome: BOUNTY_COMPLETE_ORACLE_MESSAGE.to_string(),
                        payout: dlc::Payout {
                            offer: 0,
                            accept: bounty_amount_sats + taker_collateral_sats,
                        },
                    },
                    // If the bounty is not completed, the maker gets the bounty
                    // back plus the taker's collateral as compensation for their
                    // time.
                    EnumerationPayout {
                        outcome: BOUNTY_INSUFFICIENT_ORACLE_MESSAGE.to_string(),
                        payout: dlc::Payout {
                            offer: bounty_amount_sats + taker_collateral_sats,
                            accept: 0,
                        },
                    },
                ],
            }),
            oracles: OracleInput {
                public_keys: vec![oracle_public_key],
                event_id: oracle_event_id,
                threshold: 1,
            },
        }],
    }
}
