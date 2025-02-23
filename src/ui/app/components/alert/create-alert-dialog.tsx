import * as React from "react";
import { z } from "zod";
import { useZodForm } from "@/hooks/use-fom";

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
import { ChooseTarget } from "./choose-target";
import type { Target } from "@/lib/entities";
import { DurationPicker } from "../time/duration-picker";
import { durationToTicks, ticksToDuration } from "@/lib/time";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const reminderSchema = z.object({
  threshold: z.number().min(0).max(1),
  message: z.string().min(1, "Name is required"),
});

const formSchema = z.object({
  target: z.discriminatedUnion("tag", [
    z.object({ tag: z.literal("App"), id: z.number().int() }),
    z.object({ tag: z.literal("Tag"), id: z.number().int() }),
  ]),
  usage_limit: z.number().int(),
  time_frame: z.enum(["Daily", "Weekly", "Monthly"]),
  trigger_action: z.discriminatedUnion("tag", [
    z.object({ tag: z.literal("Kill") }),
    z.object({ tag: z.literal("Dim"), duration: z.number().int() }),
    z.object({
      tag: z.literal("Message"),
      content: z.string().min(1, "Message is required"),
    }),
  ]),
  reminders: reminderSchema.array(),
});

type FormValues = z.infer<typeof formSchema>;

interface CreateAlertDialogProps {
  onSubmit: (values: FormValues) => Promise<void>;
  trigger?: React.ReactNode;
}

function targetValue(value: FormValues["target"]): Target {
  // this is correct, had to write this since
  // I was getting `number is not assignable to Ref<T>`
  return value as unknown as Target;
}

export function CreateAlertDialog({
  onSubmit,
  trigger,
}: CreateAlertDialogProps) {
  const [open, setOpen] = React.useState(false);

  const form = useZodForm({
    schema: formSchema,
    // assign default values if necessary
  });

  const handleSubmit = async (values: FormValues) => {
    await onSubmit(values);
    setOpen(false);
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || <Button variant="outline">Create Alert</Button>}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create Alert</DialogTitle>
          <DialogDescription>
            Create a new alert to monitor your usage.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(handleSubmit)}
            className="space-y-4"
          >
            <FormField
              control={form.control}
              name="target"
              render={({ field: { value, onChange, ...field } }) => (
                <FormItem>
                  <FormLabel>Target</FormLabel>
                  <FormControl>
                    {/* BUG: Can't scroll inside here??? */}
                    <ChooseTarget
                      {...field}
                      value={targetValue(value)}
                      onValueChanged={onChange}
                      className="w-full justify-start"
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="usage_limit"
              render={({ field: { value, onChange, ...field } }) => (
                <FormItem>
                  <FormLabel>Usage Limit</FormLabel>
                  <FormControl>
                    <DurationPicker
                      showIcon={false}
                      className="w-full text-foreground"
                      {...field}
                      duration={
                        value === undefined ? undefined : ticksToDuration(value)
                      }
                      setDuration={(dur) =>
                        onChange(
                          dur === undefined ? undefined : durationToTicks(dur),
                        )
                      }
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="time_frame"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Time Frame</FormLabel>
                  <FormControl>
                    <Select
                      {...field}
                      value={field.value}
                      onValueChange={field.onChange}
                    >
                      <SelectTrigger className="hover:bg-muted">
                        <SelectValue placeholder="Time Frame" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="Daily">Daily</SelectItem>
                        <SelectItem value="Weekly">Weekly</SelectItem>
                        <SelectItem value="Monthly">Monthly</SelectItem>
                      </SelectContent>
                    </Select>
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
