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
  TimelineContent,
  TimelineIndicator,
  TimelineTitle,
  TimelineSeparator,
  TimelineHeader,
  TimelineItem,
} from "@/components/ui/timeline";
import { Button } from "@/components/ui/button";
import { useCallback } from "react";
import { useAppState } from "@/lib/state";
import { useNavigate } from "react-router";

type FormValues = z.infer<typeof alertSchema>;

export default function CreateAlerts() {
  const createAlert = useAppState((state) => state.createAlert);
  const navigate = useNavigate();
  const onSubmit = useCallback(
    async (values: FormValues) => {
      await createAlert(values);
      navigate("/alerts");
    },
    [createAlert, navigate],
  );
  return (
    <>
      <main className="flex flex-1">
        <CreateAlertForm onSubmit={onSubmit} />
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
      title: "Limit",
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
                <FormLabel>Limit</FormLabel>
                <FormControl>
                  <DurationPicker
                    showIcon={false}
                    className="w-full text-muted-foreground"
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
                    onValueChange={(v) => {
                      // Reset the form when changing action type
                      if (v === "Kill") {
                        field.onChange({ tag: v });
                      } else if (v === "Dim") {
                        field.onChange({ tag: v, duration: undefined });
                      } else if (v === "Message") {
                        field.onChange({ tag: v, content: "" });
                      }
                    }}
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
    {
      id: 4,
      title: "Reminders",
      content: (
        <FormField
          control={form.control}
          name="reminders"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <div className="space-y-2">
                  {field.value.map((reminder, index) => (
                    <div key={index} className="flex gap-2 items-end">
                      <Input
                        type="number"
                        min={0}
                        max={1}
                        step={0.01}
                        placeholder="Threshold (0-1)"
                        value={reminder.threshold}
                        className="w-24"
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
      ),
    },
  ];

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="w-[360px] space-y-4 px-8 m-auto py-8"
        >
          <Timeline value={0}>
            {items.map((item) => (
              <TimelineItem key={item.id} step={item.id}>
                <TimelineHeader>
                  <TimelineSeparator className="bg-primary/60 mt-2.5 group-data-[orientation=vertical]/timeline:h-[calc(100%-2rem-0.25rem)]" />
                  <TimelineTitle className="-mt-1/2 text-base">
                    {item.title}
                  </TimelineTitle>
                  <TimelineIndicator className="border-primary/80 group-data-completed/timeline-item:bg-primary group-data-completed/timeline-item:text-primary-foreground flex size-6 items-center justify-center group-data-completed/timeline-item:border-none group-data-[orientation=vertical]/timeline:-left-6" />
                </TimelineHeader>
                <TimelineContent className="mt-2 space-y-4">
                  {item.content}
                </TimelineContent>
              </TimelineItem>
            ))}
          </Timeline>

          <div className="flex justify-end mt-4">
            <Button type="submit">Submit</Button>
          </div>
        </form>
      </Form>
    </>
  );
}
