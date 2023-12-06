// Hardcoded list of relays to publish Nostr notes to.
// TODO: Add more relays.
// TODO: Change this to DEFAULT_RELAYS and use it to
// initialize a list of relays that the user can modify.
export const RELAYS = ["wss://relay.damus.io", "wss://nos.lol"];

export enum ContractStatus {
  opened = "Opened",
  sent = "Sent",
  complete = "Complete",
}

export enum StepperStatus {
  current = "current",
  upcoming = "upcoming",
  complete = "complete",
}

export interface IContract {
  title: string;
  description: string;
  status: string;
  expiration: string;
  cost: number;
  createdOn: string;
  id: string;
  author: string;
}
