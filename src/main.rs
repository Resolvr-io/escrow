mod bdk_dlc_blockchain;
mod bdk_dlc_wallet;
mod resolvr_oracle;

use bdk_dlc_blockchain::BdkDlcBlockchain;
use bdk_dlc_wallet::BdkDlcWallet;
use bdk_electrum::electrum_client::Client;
use dlc_manager::SystemTimeProvider;
use std::collections::hash_map::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

pub type ResolvrDlcManager = dlc_manager::manager::Manager<
    Arc<BdkDlcWallet>,
    Arc<BdkDlcBlockchain>,
    Box<dlc_sled_storage_provider::SledStorageProvider>,
    Arc<resolvr_oracle::ResolvrOracle>,
    Arc<SystemTimeProvider>,
    Arc<BdkDlcBlockchain>,
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

    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();
    let bdk_dlc_blockchain = Arc::from(BdkDlcBlockchain::new(client));

    let time_provider = Arc::new(dlc_manager::SystemTimeProvider {});

    let alice_dlc_manager: Arc<Mutex<ResolvrDlcManager>> = Arc::new(Mutex::new(
        dlc_manager::manager::Manager::new(
            Arc::from(BdkDlcWallet::new()),
            bdk_dlc_blockchain.clone(),
            Box::new(
                dlc_sled_storage_provider::SledStorageProvider::new(&alice_storage_path)
                    .expect("Error creating storage."),
            ),
            oracles.clone(),
            time_provider.clone(),
            bdk_dlc_blockchain.clone(),
        )
        .expect("Could not create manager."),
    ));

    let bob_dlc_manager: Arc<Mutex<ResolvrDlcManager>> = Arc::new(Mutex::new(
        dlc_manager::manager::Manager::new(
            Arc::from(BdkDlcWallet::new()),
            bdk_dlc_blockchain.clone(),
            Box::new(
                dlc_sled_storage_provider::SledStorageProvider::new(&bob_storage_path)
                    .expect("Error creating storage."),
            ),
            oracles,
            time_provider,
            bdk_dlc_blockchain,
        )
        .expect("Could not create manager."),
    ));
}
