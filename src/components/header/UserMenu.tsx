import { fetchProfileEvent, logout } from "~/lib/auth";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import UserAvatar from "../profile/UserAvatar";
import { Link } from "react-router-dom";
import { useEffect } from "react";
import useEventStore from "~/stores/eventStore";

type UserMenuProps = {
  pubkey: string;
};

export default function UserMenu({ pubkey }: UserMenuProps) {
  const { profileMap } = useEventStore();

  useEffect(() => {
    if (profileMap[pubkey]) {
      return;
    }

    fetchProfileEvent(pubkey);
  }, [profileMap[pubkey]]);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <UserAvatar pubkey={pubkey} />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>My Account</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link to="/settings">Settings</Link>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={logout}>Sign Out</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
