mod oracle;
mod resolvr_oracle;

use bitcoin::XOnlyPublicKey;
use bitcoin_rpc_provider::BitcoinCoreProvider;
use dlc::EnumerationPayout;
use dlc_manager::contract::contract_input::{ContractInput, ContractInputInfo, OracleInput};
use dlc_manager::contract::enum_descriptor::EnumDescriptor;
use dlc_manager::contract::{Contract, ContractDescriptor};
use dlc_manager::Oracle;
use dlc_manager::Storage;
use dlc_manager::SystemTimeProvider;
use dlc_manager::Wallet;
use dlc_messages::contract_msgs::ContractInfo;
use resolvr_oracle::{
    ResolvrOracle, BOUNTY_COMPLETE_ORACLE_MESSAGE, BOUNTY_INSUFFICIENT_ORACLE_MESSAGE,
};
use std::collections::hash_map::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

struct BitcoinCoreConfig {
    host: String,
    port: u16,
    rpc_user: String,
    rpc_password: String,
}

pub type ResolvrDlcManager<'a> = dlc_manager::manager::Manager<
    Arc<BitcoinCoreProvider>,
    Arc<BitcoinCoreProvider>,
    Box<dlc_sled_storage_provider::SledStorageProvider>,
    Arc<ResolvrOracle>,
    Arc<SystemTimeProvider>,
    Arc<BitcoinCoreProvider>,
>;

#[tokio::main]
async fn main() {
    let alice_storage_path = format!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        "data/alice"
    );
    let bob_storage_path = format!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        "data/bob"
    );
    let oracle_storage_path = format!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        "data/oracle"
    );
    fs::create_dir_all(&alice_storage_path).expect("Error creating storage directory.");
    fs::create_dir_all(&bob_storage_path).expect("Error creating storage directory.");
    fs::create_dir_all(&oracle_storage_path).expect("Error creating storage directory.");
    // let offers_path = format!("{}/{}", config.storage_dir_path, "offers");
    // fs::create_dir_all(&offers_path).expect("Error creating offered contract directory");

    // Instantiate an oracle client. At the moment the implementation of the oracle
    // client uses reqwest in blocking mode to satisfy the non async oracle interface
    // so we need to use `spawn_blocking`.
    let mut oracles: HashMap<bitcoin::XOnlyPublicKey, Arc<ResolvrOracle>> = HashMap::new();
    let oracle = Arc::from(ResolvrOracle::new(&oracle_storage_path));
    oracles.insert(oracle.get_public_key(), oracle.clone());

    // TODO: Load configs from file/env vars/command line.
    let bob_config = BitcoinCoreConfig {
        host: String::from("127.0.0.1"),
        port: 18447,
        rpc_user: String::from("polaruser"),
        rpc_password: String::from("polarpass"),
    };
    let alice_config = BitcoinCoreConfig {
        host: String::from("127.0.0.1"),
        port: 18448,
        rpc_user: String::from("polaruser"),
        rpc_password: String::from("polarpass"),
    };

    let alice_bitcoind_provider = Arc::new(
        bitcoin_rpc_provider::BitcoinCoreProvider::new(
            alice_config.host.clone(),
            alice_config.port,
            None,
            alice_config.rpc_user.clone(),
            alice_config.rpc_password.clone(),
        )
        .expect("Error creating BitcoinCoreProvider"),
    );

    let bob_bitcoind_provider = Arc::new(
        bitcoin_rpc_provider::BitcoinCoreProvider::new(
            bob_config.host,
            bob_config.port,
            None,
            bob_config.rpc_user,
            bob_config.rpc_password,
        )
        .expect("Error creating BitcoinCoreProvider"),
    );

    let time_provider = Arc::new(dlc_manager::SystemTimeProvider {});

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let alice_public_key = alice_bitcoind_provider
        .get_new_secret_key()
        .unwrap()
        .public_key(&secp);
    let bob_public_key = bob_bitcoind_provider
        .get_new_secret_key()
        .unwrap()
        .public_key(&secp);

    println!("Creating Alice's DLC manager...");
    let alice_dlc_manager: Arc<Mutex<ResolvrDlcManager>> = Arc::new(Mutex::new(
        dlc_manager::manager::Manager::new(
            alice_bitcoind_provider.clone(),
            alice_bitcoind_provider.clone(),
            Box::new(
                dlc_sled_storage_provider::SledStorageProvider::new(&alice_storage_path)
                    .expect("Error creating storage."),
            ),
            oracles.clone(),
            time_provider.clone(),
            alice_bitcoind_provider,
        )
        .expect("Could not create manager."),
    ));

    println!("Creating Bob's DLC manager...");
    let bob_dlc_manager: Arc<Mutex<ResolvrDlcManager>> = Arc::new(Mutex::new(
        dlc_manager::manager::Manager::new(
            bob_bitcoind_provider.clone(),
            bob_bitcoind_provider.clone(),
            Box::new(
                dlc_sled_storage_provider::SledStorageProvider::new(&bob_storage_path)
                    .expect("Error creating storage."),
            ),
            oracles,
            time_provider,
            bob_bitcoind_provider,
        )
        .expect("Could not create manager."),
    ));

    let oracle_event_id = oracle.create_announcement();
    let dlc_contract = create_bounty_contract(
        1_000,
        0,
        10,
        oracle.get_public_key(),
        oracle_event_id.to_string(),
    );
    dlc_contract.validate().unwrap();

    let offer_dlc = alice_dlc_manager
        .lock()
        .unwrap()
        .send_offer(&dlc_contract, bob_public_key)
        .unwrap();
    match &offer_dlc.contract_info {
        ContractInfo::SingleContractInfo(s) => s.contract_info.oracle_info.validate(&secp).unwrap(),
        ContractInfo::DisjointContractInfo(_) => {
            panic!("Disjoint contract info not supported.");
        }
    }

    let min_timeout_interval = 0;
    let max_timeout_interval = 604800;

    offer_dlc
        .validate(&secp, min_timeout_interval, max_timeout_interval)
        .unwrap();
    bob_dlc_manager
        .lock()
        .unwrap()
        .on_dlc_message(&dlc_messages::Message::Offer(offer_dlc), alice_public_key)
        .unwrap();
    let contract_offers = bob_dlc_manager
        .lock()
        .unwrap()
        .get_store()
        .get_contract_offers()
        .unwrap();
    let (contract_id, _counterparty_pub_key, accept_dlc) = bob_dlc_manager
        .lock()
        .unwrap()
        .accept_contract_offer(&contract_offers.first().unwrap().id)
        .unwrap();
    let sign_dlc_message = alice_dlc_manager
        .lock()
        .unwrap()
        .on_dlc_message(&dlc_messages::Message::Accept(accept_dlc), bob_public_key)
        .unwrap()
        .unwrap();
    bob_dlc_manager
        .lock()
        .unwrap()
        .on_dlc_message(&sign_dlc_message, alice_public_key)
        .unwrap();
    alice_dlc_manager.lock().unwrap().periodic_check().unwrap();
    bob_dlc_manager.lock().unwrap().periodic_check().unwrap();
    let alice_contract = alice_dlc_manager
        .lock()
        .unwrap()
        .get_store()
        .get_contract(&contract_id)
        .expect("Error retrieving contract list.");
    let bob_contract = bob_dlc_manager
        .lock()
        .unwrap()
        .get_store()
        .get_contract(&contract_id)
        .expect("Error retrieving contract list.");
    println!("Alice Contract: {:#?}", alice_contract);
    println!("Bob Contract: {:#?}", bob_contract);

    println!("Waiting for funding confirmation...");
    loop {
        alice_dlc_manager.lock().unwrap().periodic_check().unwrap();
        bob_dlc_manager.lock().unwrap().periodic_check().unwrap();
        let alice_contract = alice_dlc_manager
            .lock()
            .unwrap()
            .get_store()
            .get_contract(&contract_id)
            .expect("Error retrieving contract list.");
        let bob_contract = bob_dlc_manager
            .lock()
            .unwrap()
            .get_store()
            .get_contract(&contract_id)
            .expect("Error retrieving contract list.");
        if let Contract::Confirmed(_) = alice_contract.unwrap() {
            break;
        }
        if let Contract::Confirmed(_) = bob_contract.unwrap() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    println!("Funding confirmed!");
    alice_dlc_manager.lock().unwrap().periodic_check().unwrap();
    bob_dlc_manager.lock().unwrap().periodic_check().unwrap();
    oracle
        .create_attestation(
            &oracle_event_id,
            resolvr_oracle::BountyOutcome::Insufficient,
        )
        .unwrap();
    loop {
        alice_dlc_manager.lock().unwrap().periodic_check().unwrap();
        bob_dlc_manager.lock().unwrap().periodic_check().unwrap();
        let alice_contract = alice_dlc_manager
            .lock()
            .unwrap()
            .get_store()
            .get_contract(&contract_id)
            .expect("Error retrieving contract list.");
        let bob_contract = bob_dlc_manager
            .lock()
            .unwrap()
            .get_store()
            .get_contract(&contract_id)
            .expect("Error retrieving contract list.");
        println!("Alice Contract: {:#?}", alice_contract);
        println!("Bob Contract: {:#?}", bob_contract);
        match alice_contract.unwrap() {
            Contract::Confirmed(_) => {}
            _ => break,
        }
        match bob_contract.unwrap() {
            Contract::Confirmed(_) => {}
            _ => break,
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

/// Create a DLC contract template for a bounty.
fn create_bounty_contract(
    bounty_amount_sats: u64,
    taker_collateral_sats: u64,
    fee_rate_sats_per_vbyte: u64,
    oracle_public_key: XOnlyPublicKey,
    oracle_bounty_event_id: String,
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
                event_id: oracle_bounty_event_id,
                threshold: 1,
            },
        }],
    }
}
