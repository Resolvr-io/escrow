import { type Event } from "nostr-tools";
import { create } from "zustand";
import { devtools } from "zustand/middleware";

export interface EventState {
  profileMap: Record<string, Event | null>;
  addProfile: (pubkey: string, profileEvent: Event | null) => void;
}

const useEventStore = create<EventState>()(
  devtools((set) => ({
    profileMap: {},
    addProfile: (pubkey, userEvent) =>
      set((prev) => {
        if (!userEvent) {
          return {};
        }
        const currentEvent = prev.profileMap[pubkey];
        if (!currentEvent || userEvent.created_at > currentEvent.created_at) {
          return {
            profileMap: {
              ...prev.profileMap,
              [pubkey]: userEvent,
            },
          };
        }
        return {};
      }),
  })),
);

export default useEventStore;
