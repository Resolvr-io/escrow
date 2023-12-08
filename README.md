# Resolvr Escrow

Resolvr Escrow is a desktop app built on [Tauri](https://tauri.app/), with a frontend powered by [React](https://react.dev/) and a backend powered by [Rust](https://www.rust-lang.org/). Currently this app handles coordination, execution, and resolution of Bitcoin escrow contracts. We plan on eventually supporting all features currently accessible on our [bounty website](https://resolvr-io.vercel.app/) within this app as well.

## Recommended IDE Setup

For the best experience, use [VS Code](https://code.visualstudio.com/) with the following plugins:

* [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
* [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

We are using `prettier` for code formatting. You can install the VS Code plugin [here](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) and enable the `Format On Save` option in your VS Code settings.

## Running in development mode

### Prerequisites

You will need to have Rust and NodeJS/Bun installed to run the app. You can install Rust [here](https://www.rust-lang.org/tools/install), NodeJS [here](https://nodejs.org/en/download/), and Bun [here](https://bun.sh/).

### Commands

1. Install JS/TS dependencies by running `bun i`.
2. Start the app by running `bun run tauri dev`. This enables hot module reloading for the React app.

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
Perhaps you're asking yourself, how does putting all the trust in an escrow agent solve the problem? Aren't we just shifting the trust from one party to another? There are two reasons why introducing this third entity can be beneficial:

1. **Concentrated Reputation**: Bounties are costly for makers to offer (since they are offering up sats) and for takers to complete (since they must expend time and mental effort). This makes the process of accruing reputation slow and inefficient. And if a maker or taker stops using the Resolvr platform, their reputation is effectively lost. We expect that escrow agents will likely adjudicate many bounties - many more than a typical maker or taker will ever be able to engage in - and build up reputation that is long-lasting and efficiently reused across many bounties. And since verifying whether work was completed is much cheaper than actually doing the work (like Bitcoin proof-of-work), an escrow agent will be able to accrue reputation much faster and more efficiently than a maker or taker could. In practice, this should mean that makers and takers that are new to the Resolvr platform and/or have low reputation will essentially be able to leverage the reputation that past makers and takers have helped escrow agents accrue in order to bootstrap their own reputations by using escrow with their bounties.

2. In the future, Resolvr will allow for Fedimint federations to act as federated escrow agents, effectively eliminating the escrow agent as a single point of failure. See the [future work](#future-work) section for details.

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

## Future Work

* **Disk Encryption**: Currently all data is stored unencrypted. It might make sense to encrypt all disk data using the user's nSec.

* **Unified App**: Since Tauri essentially provides a browser tab to use as the UI, we could eventually merge the codebases of the desktop app with our [bounty board](https://resolvr-io.vercel.app/) and provide a unified experience on both (like [Electron](https://www.electronjs.org/) apps such as Discord and Spotify) and enable "power user" features such as escrow only on the desktop app.

* **Acting as an Escrow Agent in the App**: Currently all app users must use Resolvr's hosted escrow agent. In the future, users could be able to act as escrow agents directly in the app.

* **Federated Escrow Agents**: Centralized escrow agents are a potential point of failure/corruption. We propose Fedimint as a method for running decentralized escrow agents. Fedimint is a system for running federated applications through entities called federations. Federations are able to perform actions by the federation nodes reaching consensus. This allows for any individual node to be offline or even malicious and yet unable to disrupt the rest of the nodes in the federation. We're currently building a module for the Fedimint platform that allows for federated adjudication of escrow contracts using [FROST](https://glossary.blockstream.com/frost/). Any federation running this module will be able to act as a Resolvr escrow agent. This work is being done outside of the app codebase but will provide significant impact to users of the app and so is worth mentioning here.

## Notes, TODOs, and questions to be answered

TODO: Describe the danger in taker-rugpulling and why it makes sense to always use the escrow agent for settling contracts rather than only in the case of a dispute.

Where does this fit in?
For Resolvr Escrow, both the bounty maker and taker would be required to agree on bounty terms

How will escrow help me resolve disputes?

Where is the code hosted?
* ~~Microservice (rocket?)~~
* ~~Self-hosted~~ 
* ~~In the browser (WASM)~~
* Client-side with nostr for node-to-node communication.

Why use this stack?

What are DLCs/oracle/fedimint?
