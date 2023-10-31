mod resolvr_oracle;

use bitcoin_rpc_provider::BitcoinCoreProvider;
use dlc_manager::SystemTimeProvider;
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
