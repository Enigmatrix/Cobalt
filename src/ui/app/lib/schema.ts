import { z } from "zod";
import type { Ref, App, Tag, Reminder, Alert } from "@/lib/entities";
import { Duration } from "luxon";
import { ticksToDuration } from "@/lib/time";

export function refSchema<T>() {
  return z.number().int() as unknown as z.ZodType<Ref<T>>;
}

export const durationSchema = z.number().int();

export const tagSchema = z.object({
  name: z.string().min(1, "Name is required"),
  color: z.string().regex(/^#[0-9A-Fa-f]{6}$/, "Invalid color format"),
  apps: refSchema<App>().array(),
});

export const reminderSchema = z.object({
  id: refSchema<Reminder>().optional(),
  threshold: z.number().min(0).max(1),
  message: z.string().min(1, "Message is required"),
});

export const triggerActionSchema = z.discriminatedUnion("tag", [
  z.object({ tag: z.literal("Kill") }),
  z.object({ tag: z.literal("Dim"), duration: durationSchema }),
  z.object({
    tag: z.literal("Message"),
    content: z.string().min(1, "Message is required"),
  }),
]);

export const targetSchema = z.discriminatedUnion("tag", [
  z.object({ tag: z.literal("App"), id: refSchema<App>() }),
  z.object({ tag: z.literal("Tag"), id: refSchema<Tag>() }),
]);

export const alertSchema = z
  .object({
    id: refSchema<Alert>().optional(),
    target: targetSchema,
    usage_limit: durationSchema,
    time_frame: z.enum(["Daily", "Weekly", "Monthly"]),
    trigger_action: triggerActionSchema,
    reminders: reminderSchema.array(),
    ignore_trigger: z.boolean(),
  })
  .refine(
    (data) => {
      const maxDuration: { [key: string]: Duration } = {
        Daily: Duration.fromObject({ days: 1 }),
        Weekly: Duration.fromObject({ weeks: 1 }),
        Monthly: Duration.fromObject({ days: 28 }), // smallest number of days in a month
      };

      return (
        data.time_frame &&
        ticksToDuration(data.usage_limit) <= maxDuration[data.time_frame]
      );
    },
    {
      path: ["usage_limit"],
      message: "Usage Limit cannot exceed the Time Frame",
    },
  );
