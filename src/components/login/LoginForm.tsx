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

import { useNavigate } from "react-router-dom";

import { Input } from "~/components/ui/input";

import { getPublicKey, nip19 } from "nostr-tools";

import * as z from "zod";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "~/components/ui/form";

import { login } from "~/lib/auth";
import { saveNostrNsecToKeychain } from "~/tauriApi";

function isNsec(nsec: string) {
  try {
    const decoded = nip19.decode(nsec);
    if (decoded.type !== "nsec") {
      return false;
    }
    return true;
  } catch (e) {
    return false;
  }
}

const formSchema = z.object({
  nsec: z.string().refine(isNsec, {
    message: "Invalid nsec",
  }),
});

type LoginFormProps = {
  setFormState?: (state: "login" | "register") => void;
};

export default function LoginForm({ setFormState }: LoginFormProps) {
  const navigate = useNavigate();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      nsec: "",
    },
  });

  async function onSubmit(values: z.infer<typeof formSchema>) {
    const nsec = values.nsec;
    const sk = nip19.decode(nsec).data as string;
    const pk = getPublicKey(sk);
    const npub = nip19.npubEncode(pk);
    try {
      await saveNostrNsecToKeychain(npub, nsec);
      const pubkey = nip19.decode(npub).data as string;

      login(pubkey);

      navigate("/");
    } catch (e) {
      console.log("ERROR", e);
    }
  }

  function switchToRegister() {
    if (setFormState) {
      setFormState("register");
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card className="min-w-[24rem] max-w-md">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl">Login</CardTitle>
            {/* <CardDescription> */}
            {/*   Don't have nostr account?{" "} */}
            {/*   <span */}
            {/*     onClick={switchToRegister} */}
            {/*     className="cursor-pointer text-blue-400 hover:underline" */}
            {/*   > */}
            {/*     Create one */}
            {/*   </span> */}
            {/* </CardDescription> */}
          </CardHeader>
          <CardContent className="flex flex-col gap-y-4">
            <FormField
              control={form.control}
              name="nsec"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>nsec</FormLabel>
                  <FormControl>
                    <Input placeholder="nsec..." {...field} />
                  </FormControl>
                  <FormDescription>Your nostr secret key</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full">
              Login
            </Button>
            {/*TODO: implement login with npub*/}
            {/* <div className="relative"> */}
            {/*   <div className="absolute inset-0 flex items-center"> */}
            {/*     <span className="w-full border-t" /> */}
            {/*   </div> */}
            {/*   <div className="relative flex justify-center text-xs uppercase"> */}
            {/*     <span className="bg-background text-muted-foreground px-2"> */}
            {/*       Or continue with */}
            {/*     </span> */}
            {/*   </div> */}
            {/* </div> */}
          </CardContent>
          <CardFooter>
            {/* <Button */}
            {/*   onClick={loginWithKeychain} */}
            {/*   type="button" */}
            {/*   className="w-full" */}
            {/*   variant="outline" */}
            {/* > */}
            {/*   <LockOpen1Icon className="mr-2 h-4 w-4" /> */}
            {/*   Keychain */}
            {/* </Button> */}
          </CardFooter>
        </Card>
      </form>
    </Form>
  );
}
