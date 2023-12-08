import { Button } from "~/components/ui/button";

import { Input } from "~/components/ui/input";
import { invoke } from "@tauri-apps/api";

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
import { Link } from "react-router-dom";
import { ArrowLeftIcon } from "@radix-ui/react-icons";

const formSchema = z.object({
  host: z.string(),
  port: z.coerce.number().positive({
    message: "Port must be a positive integer",
  }),
  rpcUser: z.string(),
  rpcPassword: z.string(),
});

export default function BitconNodePage() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      host: "",
      port: 0,
      rpcUser: "",
      rpcPassword: "",
    },
  });

  async function onSubmit(values: z.infer<typeof formSchema>) {
    const bitcoinCoreConfig = {
      host: values.host,
      port: values.port,
      rpc_user: values.rpcUser,
      rpc_password: values.rpcPassword,
    };

    try {
      await invoke("connect_to_bitcoin_core", {
        bitcoinCoreConfig:  bitcoinCoreConfig,
      });
    } catch (e) {
      console.log("ERROR", e);
    }
  }

  return (
    <div className="flex-1 flex-col space-y-8 p-8 pt-6 md:flex">
      <Link
        to="/"
        className="flex w-48 items-center gap-x-2 rounded-lg bg-gray-50 px-3 py-2 text-sm font-medium text-gray-800 shadow-lg shadow-gray-900/5 ring-1 ring-gray-300 hover:bg-white dark:bg-gray-800 dark:text-gray-200 dark:ring-gray-800 dark:hover:bg-gray-700/50"
      >
        <ArrowLeftIcon className="h-4 w-4" />
        <span>Back to all contracts</span>
      </Link>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
          <FormField
            control={form.control}
            name="host"
            render={({ field }) => (
              <FormItem>
                <FormLabel>host</FormLabel>
                <FormControl>
                  <Input placeholder="http://..." {...field} />
                </FormControl>
                <FormDescription>Your node host URL</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="port"
            render={({ field }) => (
              <FormItem>
                <FormLabel>port</FormLabel>
                <FormControl>
                  <Input type="number" {...field} />
                </FormControl>
                <FormDescription>The exposed port of your node</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="rpcUser"
            render={({ field }) => (
              <FormItem>
                <FormLabel>RPC User</FormLabel>
                <FormControl>
                  <Input {...field} />
                </FormControl>
                <FormDescription>The RPC user of your node</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="rpcPassword"
            render={({ field }) => (
              <FormItem>
                <FormLabel>RPC Password</FormLabel>
                <FormControl>
                  <Input {...field} />
                </FormControl>
                <FormDescription>The RPC password of your node</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button type="submit">
            Connect
          </Button>
        </form>
      </Form>
    </div>
  );
}
