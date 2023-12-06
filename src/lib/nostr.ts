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

  const profile = JSON.parse(event.content) as Profile;

  return profile;
}
