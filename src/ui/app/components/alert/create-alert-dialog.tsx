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
import { ChooseTarget } from "@/components/alert/choose-target";
import type { Target } from "@/lib/entities";
import { DurationPicker } from "@/components/time/duration-picker";
import { durationToTicks, ticksToDuration } from "@/lib/time";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";

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
    defaultValues: {
      reminders: [],
    },
  });

  const handleSubmit = async (values: FormValues) => {
    await onSubmit(values);
    setOpen(false);
    onOpenChange(false);
  };

  const onOpenChange = (open: boolean) => {
    setOpen(open);
    if (!open) form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
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

            <FormField
              control={form.control}
              name="trigger_action"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Trigger Action</FormLabel>
                  <FormControl>
                    <Select
                      {...field}
                      value={field.value?.tag}
                      onValueChange={(v) => field.onChange({ tag: v })}
                    >
                      <SelectTrigger className="hover:bg-muted">
                        <SelectValue placeholder="Trigger Action" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="Kill">Kill</SelectItem>
                        <SelectItem value="Dim">Dim</SelectItem>
                        <SelectItem value="Message">Message</SelectItem>
                      </SelectContent>
                    </Select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="trigger_action"
              render={({ field: { value, onChange, ...field } }) => (
                <>
                  {value?.tag === "Dim" && (
                    <FormItem>
                      <FormLabel>Dim Duration</FormLabel>
                      <FormControl>
                        <DurationPicker
                          showIcon={false}
                          className="w-full text-foreground"
                          {...field}
                          duration={
                            value?.duration === undefined
                              ? undefined
                              : ticksToDuration(value.duration)
                          }
                          setDuration={(dur) =>
                            onChange({
                              tag: "Dim",
                              duration:
                                dur === undefined
                                  ? undefined
                                  : durationToTicks(dur),
                            })
                          }
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                </>
              )}
            />

            <FormField
              control={form.control}
              name="trigger_action"
              render={({ field }) => (
                <>
                  {field.value?.tag === "Message" && (
                    <FormItem>
                      <FormLabel>Message Content</FormLabel>
                      <FormControl>
                        <Input
                          {...field}
                          value={field.value.content ?? ""}
                          onChange={(e) =>
                            field.onChange({
                              tag: "Message",
                              content: e.target.value,
                            })
                          }
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                </>
              )}
            />

            <FormField
              control={form.control}
              name="reminders"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Reminders</FormLabel>
                  <FormControl>
                    <div className="space-y-2">
                      {field.value.map((reminder, index) => (
                        <div key={index} className="flex gap-2 items-end">
                          <div className="flex-1 space-y-2">
                            <Input
                              type="number"
                              min={0}
                              max={1}
                              step={0.1}
                              placeholder="Threshold (0-1)"
                              value={reminder.threshold}
                              onChange={(e) => {
                                const newValue = [...field.value];
                                newValue[index].threshold = parseFloat(
                                  e.target.value,
                                );
                                field.onChange(newValue);
                              }}
                            />
                            <Input
                              placeholder="Reminder message"
                              value={reminder.message}
                              onChange={(e) => {
                                const newValue = [...field.value];
                                newValue[index].message = e.target.value;
                                field.onChange(newValue);
                              }}
                            />
                          </div>
                          <Button
                            type="button"
                            variant="destructive"
                            size="icon"
                            onClick={() => {
                              const newValue = [...field.value];
                              newValue.splice(index, 1);
                              field.onChange(newValue);
                            }}
                          >
                            <span className="sr-only">Delete reminder</span>×
                          </Button>
                        </div>
                      ))}
                      <Button
                        type="button"
                        variant="outline"
                        onClick={() => {
                          field.onChange([
                            ...field.value,
                            { threshold: 0.5, message: "" },
                          ]);
                        }}
                      >
                        Add Reminder
                      </Button>
                    </div>
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
