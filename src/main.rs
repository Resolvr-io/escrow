mod resolvr_oracle;

use bitcoin::XOnlyPublicKey;
use bitcoin_rpc_provider::BitcoinCoreProvider;
use dlc::EnumerationPayout;
use dlc_manager::contract::contract_input::{ContractInput, ContractInputInfo, OracleInput};
use dlc_manager::contract::enum_descriptor::EnumDescriptor;
use dlc_manager::contract::ContractDescriptor;
use dlc_manager::SystemTimeProvider;
use std::collections::hash_map::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

const BOUNTY_COMPLETE_ORACLE_MESSAGE: &str = "bounty complete";
const BOUNTY_INSUFFICIENT_ORACLE_MESSAGE: &str = "bounty insufficient";

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
    Arc<resolvr_oracle::ResolvrOracle>,
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
    fs::create_dir_all(&alice_storage_path).expect("Error creating storage directory.");
    fs::create_dir_all(&bob_storage_path).expect("Error creating storage directory.");
    // let offers_path = format!("{}/{}", config.storage_dir_path, "offers");
    // fs::create_dir_all(&offers_path).expect("Error creating offered contract directory");

    // Instantiate an oracle client. At the moment the implementation of the oracle
    // client uses reqwest in blocking mode to satisfy the non async oracle interface
    // so we need to use `spawn_blocking`.
    let oracles: HashMap<bitcoin::XOnlyPublicKey, Arc<resolvr_oracle::ResolvrOracle>> =
        HashMap::new();
    // TODO: Add oracles.

    // TODO: Load config from file/env vars/command line.
    let config = BitcoinCoreConfig {
        host: String::from("127.0.0.1"),
        port: 18446,
        rpc_user: String::from("polaruser"),
        rpc_password: String::from("polarpass"),
    };

    let alice_bitcoind_provider = Arc::new(
        bitcoin_rpc_provider::BitcoinCoreProvider::new(
            config.host.clone(),
            config.port,
            Some(String::from("alice")),
            config.rpc_user.clone(),
            config.rpc_password.clone(),
        )
        .expect("Error creating BitcoinCoreProvider"),
    );

    let bob_bitcoind_provider = Arc::new(
        bitcoin_rpc_provider::BitcoinCoreProvider::new(
            config.host,
            config.port,
            Some(String::from("bob")),
            config.rpc_user,
            config.rpc_password,
        )
        .expect("Error creating BitcoinCoreProvider"),
    );

    let time_provider = Arc::new(dlc_manager::SystemTimeProvider {});

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
