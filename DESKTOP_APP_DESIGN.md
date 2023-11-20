# Resolvr Escrow Desktop App Design

## UI Flow

1. Maker and taker establish a channel of communication (they could connect through a central server, or they could each generate a new Nostr keypair, share them with each other in the Resolvr website and then have both of their desktop apps use Nostr DMs)
2. (Optional) Taker reaches out to the chosen oracle, asks it to adjudicate the bounty, and waits for its approval
3. Taker sends DLC offer message to maker
4. Maker processes DLC offer message and sends DLC accept message to taker
5. Taker processes DLC accept message and sends DLC sign message to maker
6. Maker processes DLC sign message and publishes the funding transaction to the blockchain
7. Maker and taker wait for transaction to confirm
8. (Optional) Taker submits project to escrow agent
9. Maker and taker receive signed DLC oracle event from escrow agent and can now unlock the funds

## Questions

* What communication channel should we use?
* What should the UI look like (including layout and flow)?
* How is an oracle chosen? For MVP it might make sense to just hardcode it to use the official Resolvr oracle.
* How will the desktop apps connect to the oracle?
* Should we create a simple Oracle desktop app as well? Could be something really basic which can:
* Receive adjudication requests and display them
* Allow for accepting or denying requests
* Accept files from the taker to allow for project submission
* Provide “Approve Submission” and “Reject Submission” buttons which each signs a corresponding DLC oracle attestation and sends them to the maker and taker’s desktop apps
