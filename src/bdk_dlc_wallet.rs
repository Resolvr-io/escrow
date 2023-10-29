use bitcoin::secp256k1::{PublicKey, SecretKey};
use dlc_manager::{Signer, Wallet};

pub struct BdkDlcWallet {}

impl BdkDlcWallet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Signer for BdkDlcWallet {
    fn sign_tx_input(
        &self,
        tx: &mut bitcoin::Transaction,
        input_index: usize,
        tx_out: &bitcoin::TxOut,
        redeem_script: Option<bitcoin::Script>,
    ) -> Result<(), dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_secret_key_for_pubkey(
        &self,
        pubkey: &PublicKey,
    ) -> Result<SecretKey, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }
}

impl Wallet for BdkDlcWallet {
    fn get_new_address(&self) -> Result<bitcoin::Address, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_new_secret_key(&self) -> Result<SecretKey, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_utxos_for_amount(
        &self,
        amount: u64,
        fee_rate: Option<u64>,
        lock_utxos: bool,
    ) -> Result<Vec<dlc_manager::Utxo>, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn import_address(&self, address: &bitcoin::Address) -> Result<(), dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }
}
