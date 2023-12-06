import useEventStore from "~/stores/eventStore";
import { Avatar, AvatarImage, AvatarFallback } from "~/components/ui/avatar";
import { pc } from "~/lib/nostr";
import { useEffect } from "react";
import { fetchProfileEvent } from "~/lib/auth";

type Props = {
  pubkey: string;
};

export default function UserAvatar({ pubkey }: Props) {
  const seed = pubkey;
  const BOT_AVATAR_ENDPOINT = `https://api.dicebear.com/7.x/bottts-neutral/svg?seed=${seed}`;
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
        src={pc(profileMap[pubkey]).picture || BOT_AVATAR_ENDPOINT}
      />
      <AvatarFallback>Avatar</AvatarFallback>
    </Avatar>
  );
}
