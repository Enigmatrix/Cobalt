import * as React from "react";
import { z } from "zod";
import { useZodForm } from "@/hooks/use-form";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { ColorPicker } from "@/components/color-picker";
import { tagSchema } from "@/lib/schema";
import { ChooseMultiApps } from "../app/choose-multi-apps";

type FormValues = z.infer<typeof tagSchema>;

interface CreateTagDialogProps {
  onSubmit: (values: FormValues) => Promise<void>;
  trigger?: React.ReactNode;
}

export function CreateTagDialog({ onSubmit, trigger }: CreateTagDialogProps) {
  const [open, setOpen] = React.useState(false);

  const form = useZodForm({
    schema: tagSchema,
    defaultValues: {
      name: "",
      color: "#000000",
      apps: [],
    },
  });

  const handleSubmit = async (values: FormValues) => {
    await onSubmit(values);
    setOpen(false);
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || <Button variant="outline">Create Tag</Button>}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create Tag</DialogTitle>
          <DialogDescription>
            Create a new tag to organize your apps.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(handleSubmit)}
            className="space-y-4"
          >
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Name</FormLabel>
                  <FormControl>
                    <Input placeholder="Enter tag name" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="color"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Color</FormLabel>
                  <FormControl>
                    <ColorPicker
                      className="block w-full"
                      color={field.value}
                      onChange={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="apps"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Apps</FormLabel>
                  <FormControl>
                    <ChooseMultiApps
                      placeholder="Select apps"
                      value={field.value}
                      onValueChanged={field.onChange}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="submit">Create</Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
