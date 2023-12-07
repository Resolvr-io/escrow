import useEventStore from "~/stores/eventStore";
import { Avatar, AvatarImage, AvatarFallback } from "~/components/ui/avatar";
import { pc } from "~/lib/nostr";
import { useEffect } from "react";
import { fetchProfileEvent } from "~/lib/auth";
import { BOT_AVATAR_ENDPOINT } from "~/lib/constants";

type UserAvatarProps = {
  pubkey: string;
};

export default function UserAvatar({ pubkey }: UserAvatarProps) {
  const seed = pubkey;
  const { profileMap, addProfile } = useEventStore();

  async function fetchProfile() {
    const profile = await fetchProfileEvent(pubkey);
    if (!profile) {
      return;
    }
    addProfile(pubkey, profile);
  }

  useEffect(() => {
    if (profileMap[pubkey]) {
      return;
    }

    fetchProfile();
  }, [pubkey]);

  return (
    <Avatar>
      <AvatarImage
        src={
          pc(profileMap[pubkey]).picture ||
          `${BOT_AVATAR_ENDPOINT}?seed=${seed}`
        }
      />
      <AvatarFallback>Avatar</AvatarFallback>
    </Avatar>
  );
}
