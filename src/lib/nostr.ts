import { type Profile } from "~/types";
import { SimplePool, type Event, type Filter } from "nostr-tools";
const pool = new SimplePool();

export async function publish(
  relays: string[],
  event: Event,
  onSeen: (event: Event) => void,
) {
  const pubs = pool.publish(relays, event);
  try {
    await Promise.all(pubs);
  } catch (e) {
    console.error("Error publishing event: ", e);
  }

  const publishedEvent = await pool.get(relays, {
    ids: [event.id],
  });

  if (publishedEvent) {
    onSeen(publishedEvent);
  }
}

/**
 * Converts an event object into a Profile object.
 *
 * This function takes an event, which is expected to contain a stringified JSON
 * representation of a Profile in its content property. If the event is null or undefined,
 * it returns an empty Profile object. Otherwise, it parses the content of the event
 * and returns the parsed Profile object.
 *
 * @param {Event | undefined | null} event - The event object containing the profile information.
 * @returns {Profile} The parsed profile object. Returns an empty profile object if the input is null or undefined.
 */
export function pc(event: Event | undefined | null): Profile {
  if (!event) {
    return {
      name: "",
      about: "",
      picture: "",
      banner: "",
      lud06: "",
      lud16: "",
      nip05: "",
      website: "",
    };
  }

  return JSON.parse(event.content) as Profile;
}
