import { invoke } from "@tauri-apps/api";

/**
 * Save the Nostr secret key to the keychain, indexed by the Nostr public key.
 * @param npub The Nostr public key.
 * @param nsec The Nostr secret key.
 * @returns A promise that resolves when the key is saved.
 */
export const saveNostrNsecToKeychain = async (
  npub: string,
  nsec: string,
): Promise<void> => {
  return await invoke("save_nostr_nsec_to_keychain", { npub, nsec });
};

/**
 * Get the Nostr secret key from the keychain.
 * If it doesn't exist, an error is thrown.
 * @param npub The Nostr public key.
 * @returns The Nostr secret key for the given public key.
 */
export const getNostrNsecFromKeychain = async (
  npub: string,
): Promise<string> => {
  return await invoke("get_nostr_nsec_from_keychain", { npub });
};
