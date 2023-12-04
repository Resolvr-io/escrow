import {
  Avatar,
  AvatarImage,
  AvatarFallback,
} from "../../components/ui/avatar";
import { useId } from "react";

export default function Header() {
  const RANDOM_SEED = useId();
  // Seed should be some stored user ID
  const seed = null;
  const BOT_AVATAR_ENDPOINT = `https://api.dicebear.com/7.x/bottts-neutral/svg?seed=${
    seed || RANDOM_SEED
  }`;
  return (
    <div className="border-b">
      <div className="flex h-16 items-center px-8">
        <div className="flex w-full items-center space-x-4">
          <div className="flex w-full items-center justify-between gap-x-2">
            <img
              className="h-8"
              src="https://user-images.githubusercontent.com/81318863/280472537-02cdaaa6-493c-4b58-805d-f86fd449f71d.png"
              alt="resolvr"
            />
          </div>
          <Avatar>
            <AvatarImage src={BOT_AVATAR_ENDPOINT} />
            <AvatarFallback>Avatar</AvatarFallback>
          </Avatar>
        </div>
      </div>
    </div>
  );
}
