# Resolvr Escrow

## Background
Resolvr is a Bitcoin-native dispute resolution service for FOSS bounties, enabling "makers" to post rewards for bounties and "takers" to accept and complete them. For any bounty, the maker and taker both need reasonable assurance that they won't be cheated by the other party. This assurance may be possible simply through reputation of the maker/taker or a prior relationship between the two parties. However, in the rather likely event that this trust/relationship does not exist, an escrow system can be used to provide the needed assurance to both parties by introducing a third entity, the "escrow agent". The escrow agent is given the power to adjudicate bounty completion and control the flow of funds with on-chain enforceability.

## Design
Resolvr is designed from the ground up to be decentralized and resilient, and our escrow system is no different. We've also made design choices for our escrow system that optimize for privacy, which is covered in more detail below.

### Flow of Funds
Let's assume a maker has posted a bounty, a taker has accepted it, and they've both decided to rely on an escrow. From there, the bounty follows these steps:

1. The maker and taker agree on an escrow agent, and the escrow agent agrees to adjudicate the bounty (more on that in the [Escrow Agent](#escrow-agent) section).
2. The maker and taker create an on-chain contract that grants adjudication power to the escrow agent (more on that in the [Funding Contract](#funding-contract) section).
3. The taker completes the bounty and submits their work to the escrow agent for review.
4. The escrow agent signs and publishes an event stating that the bounty was or wasn't completed.
5. This signed event is used to unlock the on-chain contract and direct the funds to the appropriate party.

### Escrow Agent
Perhaps you're asking yourself, how does putting all the trust in an escrow agent solve the problem? Aren't we just shifting the trust from one party to another? If the escrow agent was a single entity then this would be the case. However, Resolvr's escrow system uses Fedimint as the escrow agent. Fedimint is a system for running federated applications through entities called federations. Federations are able to perform actions by the federation nodes reaching consensus. This allows for any individual node to be offline or even malicious and yet unable to disrupt the rest of the nodes in the federation. We've built a module for the Fedimint platform that allows for federated adjudication of escrow contracts. Any federation running this module is able to act as a Resolvr escrow agent.

### Funding Contract
The funding contract used for Resolvr escrow is a [Discreet Log Contract](https://bitcoinops.org/en/topics/discreet-log-contracts/). While a 2-of-3 multisig could achieve similar behavior, DLCs provide a few key benefits that make them a better choice as an escrow system.

* **On-Chain Privacy:** DLCs appear on-chain as simple 2-of-2 multisigs similar to a Lightning channel, making it hard to see what's happening - or even that a DLC is being used.

* **Escrow Privacy:** DLCs protect privacy by decoupling the escrow agent from the contracts it settles. The escrow agent has no knowledge of what contracts it is resolving on-chain.

* **Favorable Custody Model:** A 2-of-3 multisig escrow service could be considered custodial since the escrow agent holds a key linked to the funds, which presents legal risk. Using a DLC removes any liability of this nature since the escrow agent holds no keys that are traceable to the contracts involved.

* **Future Scalability:** DLCs are interoperable with the Lightning Network. While Resolvr's escrow service currently only supports on-chain DLCs, it is entirely possible to lift this protocol off-chain. This would eliminate on-chain fees and further increase privacy.

### Contract resolution
Once a federation is chosen as the escrow agent for a bounty contract, it has the responsibility to deem a bounty as completed or uncompleted according to the bounty description, with the decision being backed by on-chain enforceability. The bounty description is written in natural language by the maker and interpreted by the taker and escrow agent. This allows for a lot of flexibility to specify the conditions at which a contract should be resolved or terminated. It also means that it should be extremely thorough and detailed to limit the possibility for misinterpretation.

Once a bounty submission is provided to the escrow agent by the taker, it is reviewed according to the bounty description. If the escrow agent deems the submission sufficient, it broadcasts an event signature that unlocks the DLC to the taker's address. If the escrow agent deems the submission insufficient, it has a choice to make. It can provide feedback to the taker and request changes, or it can simply broadcast an event signature that unlocks the DLC to the maker's address, effectively cancelling the contract. We currently don't have a standard for when either option is appropriate. This could be arranged in the bounty description, it could be published by the escrow agent as a standard set of rules it abides by, or it could even be coordinated by all three parties for complex or high-value bounties.

Once the escrow agent broadcasts an event signature for the bounty, that signature can be used to tweak one of the DLC spending transactions to make it valid. This tweaking can be done by the maker or the taker, and the transaction can then be broadcasted to the Bitcoin network.

#### Timeout and Expiration
When creating a bounty DLC, a timeout is included that returns all funds in the contract back to the maker if it is exceeded. This timeout is agreed to by all parties and can be hit if the escrow agent never receives a bounty submission it considers sufficient or if the escrow agent goes offline.

## Notes, TODOs, and questions to be answered

TODO: Describe the danger in taker-rugpulling and why it makes sense to always use the escrow agent for settling contracts rather than only in the case of a dispute.

TODO: Figure out whether we even need a "contract unresolved" DLC outcome. Could we just use the built-in DLC expiration?

What's a PSBT?
A PSBT is a Partially Signed Bitcoin Transaction. It is a format for transactions that are not fully signed and allows for transactions to be passed around multiple parties until all necessary signatures are collected. Its purpose is to standardize the way different wallets and services handle the signing of transactions, especially in situations where multiple signatures are required, such as with multi-signature wallets. It enhances the flexibility and security of Bitcoin transactions.

Where does this fit in?
For Resolvr Escrow, both the bounty maker and taker would be required to agree on bounty terms and sign a PSBT broadcast by our service

How will escrow help me resolve disputes?

Where is the code hosted?
* Microservice (rocket?)
* Self-hosted 
* In the browser (WASM)

Why use this stack?

What are DLCs/oracle/fedimint?

Couldn't the oracle be compromised?
By using a federated signing process, frost, we remove a single point of failure.
A fedimint allows us to act as a singular entity.