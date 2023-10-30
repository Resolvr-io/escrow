use bdk_electrum::{
    electrum_client::{Client, ElectrumApi},
    ElectrumExt,
};
use dlc_manager::Blockchain;
use lightning::chain::chaininterface::FeeEstimator;

pub struct BdkDlcBlockchain {
    client: Client,
}

impl BdkDlcBlockchain {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Blockchain for BdkDlcBlockchain {
    fn send_transaction(
        &self,
        transaction: &bitcoin::Transaction,
    ) -> Result<(), dlc_manager::error::Error> {
        self.client
            .transaction_broadcast(transaction)
            .map_err(|e| dlc_manager::error::Error::BlockchainError(e.to_string()))?;
        Ok(())
    }

    fn get_network(
        &self,
    ) -> Result<bitcoin::network::constants::Network, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_blockchain_height(&self) -> Result<u64, dlc_manager::error::Error> {
        Ok(self
            .client
            .get_tip()
            .map_err(|e| dlc_manager::error::Error::BlockchainError(e.to_string()))?
            .0 as u64)
    }

    fn get_block_at_height(
        &self,
        height: u64,
    ) -> Result<bitcoin::Block, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }

    fn get_transaction(
        &self,
        tx_id: &bitcoin::Txid,
    ) -> Result<bitcoin::Transaction, dlc_manager::error::Error> {
        Ok(self
            .client
            .transaction_get(tx_id)
            .map_err(|e| dlc_manager::error::Error::BlockchainError(e.to_string()))?)
    }

    fn get_transaction_confirmations(
        &self,
        tx_id: &bitcoin::Txid,
    ) -> Result<u32, dlc_manager::error::Error> {
        // TODO: Implement.
        panic!("Not implemented.");
    }
}

impl FeeEstimator for BdkDlcBlockchain {
    fn get_est_sat_per_1000_weight(
        &self,
        confirmation_target: lightning::chain::chaininterface::ConfirmationTarget,
    ) -> u32 {
        // TODO: Implement.
        panic!("Not implemented.");
    }
}
