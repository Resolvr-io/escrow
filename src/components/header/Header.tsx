import { Link } from "react-router-dom";
import LoginButton from "./LoginButton";
import useAuthStore from "~/stores/authStore";
import UserMenu from "./UserMenu";

export default function Header() {
  const { pubkey } = useAuthStore();

  return (
    <div className="border-b">
      <div className="flex h-16 items-center px-8">
        <div className="flex w-full items-center space-x-4">
          <div className="flex w-full items-center justify-between gap-x-2">
            <Link to="/">
              <img
                className="h-8"
                src="https://user-images.githubusercontent.com/81318863/280472537-02cdaaa6-493c-4b58-805d-f86fd449f71d.png"
                alt="resolvr"
              />
            </Link>
          </div>
          {pubkey ? <UserMenu pubkey={pubkey} /> : <LoginButton />}
        </div>
      </div>
    </div>
  );
}
