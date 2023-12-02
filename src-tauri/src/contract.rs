use dlc_manager::contract::accepted_contract::AcceptedContract;
use dlc_manager::contract::offered_contract::OfferedContract;
use dlc_manager::contract::ser::Serializable;
use dlc_manager::contract::signed_contract::SignedContract;
use dlc_manager::contract::{
    ClosedContract, Contract, FailedAcceptContract, FailedSignContract, PreClosedContract,
};
use dlc_manager::error::Error;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Serialize, Deserialize)]
pub struct JsonContract {
    id: String,

    counter_party_id: String,

    /// Whether the local party is the original offerer of the contract.
    /// Is only None for closed contracts.
    is_offer_party: Option<bool>,

    stage: JsonContractStage,
}

impl From<&Contract> for JsonContract {
    fn from(contract: &Contract) -> Self {
        let stage = match contract {
            Contract::Offered(_) => JsonContractStage::Offered,
            Contract::Accepted(_) => JsonContractStage::Accepted,
            Contract::Signed(_) => JsonContractStage::Signed,
            Contract::Confirmed(_) => JsonContractStage::Confirmed,
            Contract::PreClosed(_) => JsonContractStage::PreClosed,
            Contract::Closed(_) => JsonContractStage::Closed,
            Contract::FailedAccept(_) => JsonContractStage::FailedAccept,
            Contract::FailedSign(_) => JsonContractStage::FailedSign,
            Contract::Refunded(_) => JsonContractStage::Refunded,
            Contract::Rejected(_) => JsonContractStage::Rejected,
        };

        JsonContract {
            id: hex::encode(contract.get_id()),
            counter_party_id: contract.get_counter_party_id().to_string(),
            is_offer_party: is_offer_party(contract),
            stage,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum JsonContractStage {
    Offered,
    Accepted,
    Signed,
    Confirmed,
    PreClosed,
    Closed,
    Refunded,
    FailedAccept,
    FailedSign,
    Rejected,
}

fn is_offer_party(contract: &Contract) -> Option<bool> {
    match contract {
        Contract::Offered(c) | Contract::Rejected(c) => Some(c.is_offer_party),
        Contract::Accepted(c) => Some(c.offered_contract.is_offer_party),
        Contract::Signed(c) | Contract::Confirmed(c) | Contract::Refunded(c) => {
            Some(c.accepted_contract.offered_contract.is_offer_party)
        }
        Contract::PreClosed(c) => Some(
            c.signed_contract
                .accepted_contract
                .offered_contract
                .is_offer_party,
        ),
        Contract::Closed(_) => None,
        Contract::FailedAccept(c) => Some(c.offered_contract.is_offer_party),
        Contract::FailedSign(c) => Some(c.accepted_contract.offered_contract.is_offer_party),
    }
}
