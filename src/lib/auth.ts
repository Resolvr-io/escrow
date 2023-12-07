import { type Filter, type Event, SimplePool } from "nostr-tools";
import { Store } from "tauri-plugin-store";
import useAuthStore from "~/stores/authStore";
import useEventStore from "~/stores/eventStore";
import { RELAYS } from "./constants";

const store = new Store(".credentials.dat");

const pool = new SimplePool();

const { setPubkey } = useAuthStore.getState();

const { addProfile } = useEventStore.getState();

export async function fetchProfileEvent(pubkey: string): Promise<Event | null> {
  const filter: Filter = {
    kinds: [0],
    authors: [pubkey],
  };
  const profileEvent = await pool.get(RELAYS, filter);
  if (profileEvent) {
    addProfile(pubkey, profileEvent);
  }
  return profileEvent;
}

export async function login(pubkey: string) {
  await store.set("pubkey", pubkey);
  await store.save();
  setPubkey(pubkey);
  const profileEvent = await fetchProfileEvent(pubkey);
  if (profileEvent) {
    addProfile(pubkey, profileEvent);
  }
}

export async function restoreLogin() {
  const pubkey: string | null = await store.get("pubkey");
  if (pubkey) {
    setPubkey(pubkey);
  }
}

export async function logout() {
  await store.delete("pubkey");
  setPubkey("");
}
