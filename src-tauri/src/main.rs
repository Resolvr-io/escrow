// Prevents additional console window on Windows in release. DO NOT REMOVE!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod contract;
mod helper;
mod nostr;
mod resolvr_oracle;

use bitcoin_rpc_provider::BitcoinCoreProvider;
use contract::JsonContract;
use dlc::EnumerationPayout;
use dlc_manager::contract::contract_input::{ContractInput, ContractInputInfo, OracleInput};
use dlc_manager::contract::enum_descriptor::EnumDescriptor;
use dlc_manager::contract::{Contract, ContractDescriptor};
use dlc_manager::Oracle;
use dlc_manager::Storage;
use dlc_manager::SystemTimeProvider;
use dlc_messages::Message;
use dlc_sled_storage_provider::SledStorageProvider;
use escrow_agent_messages::EscrowAgent;
use escrow_agent_messages::{AdjudicationRequest, AdjudicationRequestStatus};
use helper::{bitcoin_xonly_to_nostr_xonly, nostr_xonly_to_bitcoin_xonly};
use keyring::Entry;
use nostr::{NostrNip04MessageProvider, ResolvrEscrowMessage};
use resolvr_oracle::{
    NostrNip4ResolvrOracle, BOUNTY_COMPLETE_ORACLE_MESSAGE, BOUNTY_INSUFFICIENT_ORACLE_MESSAGE,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

static RESOLVR_KEYRING_SERVICE: &str = "resolvr";

#[tauri::command]
async fn save_nostr_nsec_to_keychain(
    npub: &str,
    nsec: &str,
    msg_provider_or: tauri::State<'_, Arc<Mutex<Option<NostrNip04MessageProvider>>>>,
) -> Result<(), String> {
    init_msg_provider(nsec, msg_provider_or).await;
    let entry = Entry::new(RESOLVR_KEYRING_SERVICE, npub).map_err(|e| e.to_string())?;
    entry.set_password(nsec).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_nostr_nsec_from_keychain(
    npub: &str,
    msg_provider_or: tauri::State<'_, Arc<Mutex<Option<NostrNip04MessageProvider>>>>,
) -> Result<String, String> {
    let entry = Entry::new(RESOLVR_KEYRING_SERVICE, npub).map_err(|e| e.to_string())?;
    let nsec = entry.get_password().map_err(|e| e.to_string())?;
    init_msg_provider(&nsec, msg_provider_or).await;

    Ok(nsec)
}

async fn init_msg_provider(
    nsec: &str,
    msg_provider_or: tauri::State<'_, Arc<Mutex<Option<NostrNip04MessageProvider>>>>,
) {
    let mut msg_provider_or = msg_provider_or.lock().await;
    let binding = msg_provider_or.deref_mut();
    if binding.is_none() {
        let secp = nostr_sdk::secp256k1::Secp256k1::new();
        *binding = Some(
            NostrNip04MessageProvider::new(
                nostr_sdk::prelude::KeyPair::from_seckey_str(&secp, &nsec)
                    .unwrap()
                    .secret_key(),
            )
            .await
            .unwrap(),
        );
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
async fn connect_to_bitcoin_core(
    bitcoin_core_config: BitcoinCoreConfig,
    oracle: tauri::State<'_, Arc<NostrNip4ResolvrOracle>>,
    dlc_storage: tauri::State<'_, Arc<SledStorageProvider>>,
    dlc_manager_or: tauri::State<'_, Arc<Mutex<Option<ResolvrDlcManager>>>>,
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

    let mut oracles: HashMap<bitcoin::XOnlyPublicKey, Arc<NostrNip4ResolvrOracle>> = HashMap::new();
    oracles.insert(oracle.get_public_key(), oracle.inner().clone());

    let mut dlc_manager_or: MutexGuard<Option<ResolvrDlcManager>> = dlc_manager_or.lock().await;
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
async fn offer_contract(
    bounty_amount_sats: u64,
    taker_collateral_sats: u64,
    fee_rate_sats_per_vbyte: u64,
    oracle_event_id: String,
    counter_party_public_key: &str,
    dlc_manager_or: tauri::State<'_, Arc<Mutex<Option<ResolvrDlcManager>>>>,
    msg_provider_or: tauri::State<'_, Arc<Mutex<Option<NostrNip04MessageProvider>>>>,
    oracle: tauri::State<'_, Arc<NostrNip4ResolvrOracle>>,
) -> Result<(), String> {
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

    let offer_dlc = {
        let mut dlc_manager_or = dlc_manager_or.lock().await;
        let mut binding = dlc_manager_or.as_mut();
        let dlc_manager = match &mut binding {
            Some(m) => m,
            None => return Err(String::from("DLC manager not initialized.")),
        };

        match dlc_manager.send_offer(&dlc_contract, public_key) {
            Ok(offer_dlc) => offer_dlc,
            Err(e) => return Err(format!("Error creating contract offer: {}", e)),
        }
    };

    match msg_provider_or.lock().await.as_mut() {
        Some(msg_provider) => {
            let (counter_party_x_only_pubkey, counter_party_parity) =
                public_key.x_only_public_key();
            msg_provider
                .send(
                    helper::bitcoin_xonly_to_nostr_xonly(&counter_party_x_only_pubkey),
                    nostr::ResolvrEscrowMessage::OfferDlc((offer_dlc, counter_party_parity)),
                )
                .await
                .unwrap();
        }
        None => return Err(String::from("Message provider not initialized.")),
    }

    Ok(())
}

#[tauri::command]
async fn accept_contract(
    contract_id: String,
    dlc_manager_or: tauri::State<'_, Arc<Mutex<Option<ResolvrDlcManager>>>>,
    msg_provider_or: tauri::State<'_, Arc<Mutex<Option<NostrNip04MessageProvider>>>>,
) -> Result<JsonContract, String> {
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

    let (counter_party, accept_dlc, contract) = {
        let mut dlc_manager_or = dlc_manager_or.lock().await;
        let mut binding = dlc_manager_or.as_mut();
        let dlc_manager = match &mut binding {
            Some(m) => m,
            None => return Err(String::from("DLC manager not initialized.")),
        };

        let (_, counter_party, accept_dlc) = match dlc_manager.accept_contract_offer(&contract_id) {
            Ok(res) => res,
            Err(e) => return Err(format!("Error accepting contract: {}", e)),
        };

        let contract = match dlc_manager.get_store().get_contract(&contract_id) {
            Ok(v) => match v {
                Some(v) => v,
                None => return Err(String::from("Contract not found.")),
            },
            Err(e) => return Err(format!("Error getting contract: {}", e)),
        };

        (counter_party, accept_dlc, contract)
    };

    let (counter_party_x_only_pubkey, counter_party_parity) = counter_party.x_only_public_key();

    let mut msg_provider_or = msg_provider_or.lock().await;
    let mut binding = msg_provider_or.as_mut();
    let msg_provider = match &mut binding {
        Some(m) => m,
        None => return Err(String::from("DLC manager not initialized.")),
    };
    msg_provider
        .send(
            helper::bitcoin_xonly_to_nostr_xonly(&counter_party_x_only_pubkey),
            nostr::ResolvrEscrowMessage::AcceptDlc((accept_dlc, counter_party_parity)),
        )
        .await
        .unwrap();

    Ok((&contract).into())
}

#[tauri::command]
async fn delete_contract(
    contract_id: String,
    dlc_manager_or: tauri::State<'_, Arc<Mutex<Option<ResolvrDlcManager>>>>,
) -> Result<(), String> {
    let mut dlc_manager_or = dlc_manager_or.lock().await;
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

    let msg_provider_or: Arc<Mutex<Option<NostrNip04MessageProvider>>> =
        Arc::from(Mutex::from(None));

    let dlc_manager_or_clone = dlc_manager_or.clone();
    let dlc_msg_handler_or_clone = msg_provider_or.clone();
    tokio::task::spawn(async move {
        // Store incoming messages outside of the loop so that we can retrieve
        // incoming messages from the message provider entirely separately from
        // processing them and sending outgoing messages. This is necessary
        // because the message provider and the DLC manager are both behind
        // mutexes, and we don't want to hold both mutexes at the same time.
        let mut incoming_dlc_messages: Vec<(Message, secp256k1_zkp::PublicKey)> = Vec::new();

        // Store outgoing messages outside of the loop. See comment above.
        let mut outgoing_dlc_messages: Vec<(Message, secp256k1_zkp::PublicKey)> = Vec::new();

        loop {
            // Populate `incoming_dlc_messages` from the message provider.
            {
                let mut dlc_msg_handler_or = dlc_msg_handler_or_clone.lock().await;
                if let Some(dlc_msg_handler) = dlc_msg_handler_or.as_mut() {
                    while let Some(msg) = dlc_msg_handler.pop() {
                        let (dlc_msg, sender_parity) = msg.msg.to_dlc_message();
                        incoming_dlc_messages.push((
                            dlc_msg,
                            nostr_xonly_to_bitcoin_xonly(&msg.sender).public_key(sender_parity),
                        ));
                    }
                }
            }

            // Process `incoming_dlc_messages` and populate `outgoing_dlc_messages`.
            {
                let mut dlc_manager_or = dlc_manager_or_clone.lock().await;
                if let Some(dlc_manager) = dlc_manager_or.as_mut() {
                    while let Some((dlc_msg, sender)) = incoming_dlc_messages.pop() {
                        if let Ok(Some(dlc_msg)) = dlc_manager.on_dlc_message(&dlc_msg, sender) {
                            outgoing_dlc_messages.push((dlc_msg, sender));
                        }
                    }
                }
            }

            // Send `outgoing_dlc_messages` to the message provider.
            {
                let mut dlc_msg_handler_or = dlc_msg_handler_or_clone.lock().await;
                if let Some(dlc_msg_handler) = dlc_msg_handler_or.as_mut() {
                    while let Some((dlc_msg, sender_pubkey)) = outgoing_dlc_messages.pop() {
                        let (sender_x_only_pubkey, sender_pubkey_parity) =
                            sender_pubkey.x_only_public_key();
                        dlc_msg_handler
                            .send(
                                bitcoin_xonly_to_nostr_xonly(&sender_x_only_pubkey),
                                ResolvrEscrowMessage::from_dlc_message(
                                    dlc_msg,
                                    sender_pubkey_parity,
                                )
                                .unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_nostr_nsec_to_keychain,
            get_nostr_nsec_from_keychain,
            request_oracle_adjudication,
            get_oracle_adjudication_request_status,
            connect_to_bitcoin_core,
            get_contracts,
            offer_contract,
            accept_contract,
            delete_contract
        ])
        .manage(oracle)
        .manage(dlc_storage_provider)
        .manage(dlc_manager_or)
        .manage(msg_provider_or)
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(context)
        .expect("Error while running Tauri application.");
}

/// Create a DLC contract template for a bounty.
fn create_bounty_contract(
    bounty_amount_sats: u64,
    taker_collateral_sats: u64,
    fee_rate_sats_per_vbyte: u64,
    oracle_public_key: bitcoin::XOnlyPublicKey,
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
