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
import { invoke } from "@tauri-apps/api";

import { generatePrivateKey, getPublicKey, nip19 } from "nostr-tools";

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

import { Store } from "tauri-plugin-store";

function isNpub(npub: string) {
  try {
    const decoded = nip19.decode(npub);
    if (decoded.type !== "npub") {
      return false;
    }
    return true;
  } catch (e) {
    return false;
  }
}

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
  npub: z.string().refine(isNpub, {
    message: "Invalid npub",
  }),
  nsec: z.string().refine(isNsec, {
    message: "Invalid nsec",
  }),
});

type RegisterFormProps = {
  setFormState?: (state: "login" | "register") => void;
};

export default function RegisterForm({ setFormState }: RegisterFormProps) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      npub: "",
      nsec: "",
    },
  });

  async function onSubmit(values: z.infer<typeof formSchema>) {
    const pk = nip19.decode(values.npub).data as string;
    const npub = nip19.npubEncode(pk);
    const response: string = await invoke("login_with_npub", {
      npub: npub,
    });

    if (response !== "success") {
      return;
    }

    const store = new Store(".credentials.dat");
    await store.set("pubkey", { value: pk });
    await store.save();
  }

  function switchToLogin() {
    if (setFormState) {
      setFormState("login");
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card className="min-w-[24rem] max-w-md">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl">Register</CardTitle>
            <CardDescription>
              Already have nostr account?{" "}
              <span
                onClick={switchToLogin}
                className="cursor-pointer text-blue-400 hover:underline"
              >
                Login
              </span>
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-y-4">
            <FormField
              control={form.control}
              name="npub"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>npub</FormLabel>
                  <FormControl>
                    <Input placeholder="npub..." {...field} />
                  </FormControl>
                  <FormDescription>Your nostr public key</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
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
          </CardContent>
          <CardFooter>
            <Button type="submit" className="w-full">
              Create Account
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  );
}
