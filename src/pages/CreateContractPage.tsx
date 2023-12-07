import ArrowLeftIcon from "@bitcoin-design/bitcoin-icons-react/filled/ArrowLeftIcon";
import { Link } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "~/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { CalendarIcon } from "@radix-ui/react-icons";
import { Calendar } from "~/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "~/components/ui/popover";
import { format } from "date-fns";
import { cn } from "~/lib/utils";
import { useState } from "react";

const formSchema = z.object({
  title: z.string().min(2, {
    message: "title must be at least 2 characters.",
  }),
  description: z.string().min(2, {
    message: "Description must be at least 2 characters.",
  }),
  cost: z.coerce.number().positive({
    message: "Cost must be a positive integer",
  }),
  expiration: z.date().min(new Date(), {
    message: "Date must be a future date",
  }),
});

export default function CreateContractPage() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: "",
      description: "",
      cost: 0,
      expiration: new Date(),
    },
  });
  function onSubmit(values: z.infer<typeof formSchema>) {
    // TODO: Use form values instead of logging them.
    console.log(values);
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
            name="title"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Title</FormLabel>
                <FormControl>
                  <Input placeholder="Title" {...field} />
                </FormControl>
                <FormDescription>The title of your contract</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="description"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Description</FormLabel>
                <FormControl>
                  <Input
                    placeholder="Some descriptive text"
                    min={0}
                    {...field}
                  />
                </FormControl>
                <FormDescription>
                  A description of your contract
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="cost"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Cost</FormLabel>
                <FormControl>
                  <Input type="number" placeholder="0" {...field} />
                </FormControl>
                <FormDescription>
                  The cost (in Satoshis) of your contract
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="expiration"
            render={({ field }) => (
              <FormItem className="flex flex-col">
                <FormLabel>Expiration</FormLabel>
                <Popover>
                  <PopoverTrigger asChild>
                    <FormControl>
                      <Button
                        variant={"outline"}
                        className={cn(
                          "w-[240px] pl-3 text-left font-normal",
                          !field.value && "text-muted-foreground",
                        )}
                      >
                        {field.value ? (
                          format(field.value, "PPP")
                        ) : (
                          <span>Pick a date</span>
                        )}
                        <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                      </Button>
                    </FormControl>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0" align="start">
                    <Calendar
                      mode="single"
                      selected={field.value}
                      onSelect={field.onChange}
                      initialFocus
                    />
                  </PopoverContent>
                </Popover>
                <FormDescription>
                  A future date when the contract will expire
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button type="submit">Submit</Button>
        </form>
      </Form>
    </div>
  );
}
