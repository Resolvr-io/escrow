import { LockOpen1Icon } from "@radix-ui/react-icons";
import { Button } from "~/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";

import { generatePrivateKey, getPublicKey, nip19 } from "nostr-tools";

export default function LoginPage() {
  async function setSecret() {
    const sk = generatePrivateKey();
    const pk = getPublicKey(sk);
    const nsec = nip19.nsecEncode(sk);
    const npub = nip19.npubEncode(pk);
  }

  return (
    <div className="flex h-screen items-center justify-center">
      <Card className="min-w-[24rem] max-w-md">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl">Login</CardTitle>
          <CardDescription>Login using your keychain</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-y-4">
          <Button variant="outline">
            <LockOpen1Icon className="mr-2 h-4 w-4" />
            Access Keychain
          </Button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background text-muted-foreground px-2">
                Or continue with
              </span>
            </div>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="nsec">nsec</Label>
            <Input id="nsec" type="text" placeholder="nsec..." />
          </div>
        </CardContent>
        <CardFooter>
          <Button onClick={setSecret} className="w-full">
            Login
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
