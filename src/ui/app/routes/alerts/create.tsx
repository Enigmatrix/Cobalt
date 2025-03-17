import { useZodForm } from "@/hooks/use-form";
import { alertSchema } from "@/lib/schema";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import type { z } from "zod";
import { ChooseTarget } from "@/components/alert/choose-target";
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
import {
  Timeline,
  TimelineDate,
  TimelineContent,
  TimelineIndicator,
  TimelineTitle,
  TimelineSeparator,
  TimelineHeader,
  TimelineItem,
} from "@/components/ui/timeline";
import { Button } from "@/components/ui/button";
import type { UseFormReturn } from "react-hook-form";
import { CheckIcon } from "lucide-react";
import { useMemo } from "react";

type FormValues = z.infer<typeof alertSchema>;

export default function CreateAlerts() {
  return (
    <>
      <main className="flex flex-1">
        <CreateAlertForm onSubmit={() => {}} />
        <div className="flex-1"></div>
      </main>
    </>
  );
}

export function CreateAlertForm({
  onSubmit,
}: {
  onSubmit: (values: FormValues) => void;
}) {
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      reminders: [],
    },
  });

  const items = [
    {
      id: 1,
      title: "Target",
      completed(form: UseFormReturn<FormValues>) {
        const targetState = form.getFieldState("target");
        return targetState.isDirty && !targetState.invalid;
      },
      content: (
        <FormField
          control={form.control}
          name="target"
          render={({ field: { value, onChange, ...field } }) => (
            <FormItem>
              <FormControl>
                <ChooseTarget
                  {...field}
                  value={value}
                  onValueChanged={onChange}
                  className="w-full justify-start"
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      ),
    },
    {
      id: 2,
      title: "Duration",
      completed(form: UseFormReturn<FormValues>) {
        const timeFrameState = form.getFieldState("time_frame");
        const usageLimitState = form.getFieldState("usage_limit");
        return (
          timeFrameState.isDirty &&
          !timeFrameState.invalid &&
          usageLimitState.isDirty &&
          !usageLimitState.invalid
        );
      },
      content: (
        <>
          <FormField
            control={form.control}
            name="time_frame"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Period</FormLabel>
                <FormControl>
                  <Select
                    {...field}
                    value={field.value}
                    onValueChange={field.onChange}
                  >
                    <SelectTrigger className="hover:bg-muted w-full">
                      <SelectValue placeholder="Period" />
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
            name="usage_limit"
            render={({ field: { value, onChange, ...field } }) => (
              <FormItem>
                <FormLabel>Usage Limit</FormLabel>
                <FormControl>
                  <DurationPicker
                    showIcon={false}
                    className="w-full text-foreground"
                    {...field}
                    value={value === undefined ? null : ticksToDuration(value)}
                    onValueChange={(dur) =>
                      onChange(dur === null ? undefined : durationToTicks(dur))
                    }
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </>
      ),
    },
    {
      id: 3,
      title: "Action",
      completed(form: UseFormReturn<FormValues>) {
        const triggerActionState = form.getFieldState("trigger_action");
        return triggerActionState.isDirty && !triggerActionState.invalid;
      },
      content: (
        <>
          <FormField
            control={form.control}
            name="trigger_action"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Select
                    {...field}
                    value={field.value?.tag}
                    onValueChange={(v) => field.onChange({ tag: v })}
                  >
                    <SelectTrigger className="hover:bg-muted w-full">
                      <SelectValue placeholder="Action Type" />
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
                        value={
                          value?.duration === undefined
                            ? null
                            : ticksToDuration(value.duration)
                        }
                        onValueChange={(dur) =>
                          onChange({
                            tag: "Dim",
                            duration:
                              dur === null ? undefined : durationToTicks(dur),
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
        </>
      ),
    },
  ];
  const watcher = form.watch();
  const incompleteStep = useMemo(
    () => items.find((item) => !item.completed(form)),
    [watcher],
  );

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="min-w-[320px] space-y-4 px-8 m-auto"
        >
          <Timeline value={(incompleteStep?.id ?? 4) - 1}>
            {items.map((item) => (
              <TimelineItem key={item.id} step={item.id}>
                <TimelineHeader>
                  <TimelineSeparator className="mt-2.5 group-data-[orientation=vertical]/timeline:h-[calc(100%-2rem-0.25rem)]" />
                  <TimelineTitle className="-mt-1/2 text-base">
                    {item.title}
                  </TimelineTitle>
                  <TimelineIndicator className="group-data-completed/timeline-item:bg-primary group-data-completed/timeline-item:text-primary-foreground flex size-6 items-center justify-center group-data-completed/timeline-item:border-none group-data-[orientation=vertical]/timeline:-left-6">
                    <CheckIcon
                      className="group-not-data-completed/timeline-item:hidden"
                      size={16}
                    />
                  </TimelineIndicator>
                </TimelineHeader>
                <TimelineContent className="mt-2 space-y-4">
                  {item.content}
                </TimelineContent>
              </TimelineItem>
            ))}
          </Timeline>

          <div className="flex justify-end">
            <Button type="submit">Submit</Button>
          </div>
        </form>
      </Form>
    </>
  );
}
