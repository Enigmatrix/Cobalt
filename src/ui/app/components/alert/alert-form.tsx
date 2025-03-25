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
import type { FieldArrayWithId, UseFormReturn } from "react-hook-form";
import { TriangleAlertIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import type { TriggerInfo } from "@/hooks/use-trigger-info";

export type FormValues = z.infer<typeof alertSchema>;

export function AlertForm({
  onSubmit,
  form,
  triggerInfo,
  remindersFields: fields,
  remindersAppend: append,
  remindersRemove: remove,
}: {
  onSubmit: (values: FormValues) => void;
  form: UseFormReturn<FormValues>;
  triggerInfo: TriggerInfo;
  remindersFields: FieldArrayWithId<FormValues, "reminders", "id">[];
  remindersAppend: (value: { threshold: number; message: string }) => void;
  remindersRemove: (index: number) => void;
}) {
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
            name="timeFrame"
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
            name="usageLimit"
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
            name="triggerAction"
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
                      <SelectValue placeholder="Type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="Dim">Dim</SelectItem>
                      <SelectItem value="Message">Message</SelectItem>
                      <SelectItem value="Kill">Kill</SelectItem>
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="triggerAction"
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
            name="triggerAction"
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
      title: (
        <div className="flex">
          <div>Reminders</div>
          <Button
            variant="outline"
            size="sm"
            type="button"
            className="ml-auto"
            onClick={() => {
              append({ threshold: 0.5, message: "" });
            }}
          >
            Add
          </Button>
        </div>
      ),
      content: (
        <div className="flex flex-col gap-2">
          {fields.map((field, index) => (
            <div key={field.id} className="flex flex-col gap-2">
              <div className="flex gap-2 items-center">
                <Badge variant="outline" className="">
                  {index + 1}
                </Badge>
                <FormField
                  control={form.control}
                  name={`reminders.${index}.threshold`}
                  render={({ field }) => (
                    <FormItem>
                      <FormControl>
                        <Input
                          type="number"
                          min={0}
                          max={1}
                          step="any"
                          placeholder="Threshold (0-1)"
                          className="w-16"
                          {...field}
                          onChange={(e) =>
                            field.onChange(parseFloat(e.target.value))
                          }
                        />
                      </FormControl>
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name={`reminders.${index}.message`}
                  render={({ field }) => (
                    <FormItem className="flex-1">
                      <FormControl>
                        <Input placeholder="Reminder message" {...field} />
                      </FormControl>
                    </FormItem>
                  )}
                />
                <Button
                  type="button"
                  variant="destructive"
                  size="icon"
                  onClick={() => remove(index)}
                >
                  <span className="sr-only">Delete reminder</span>×
                </Button>
              </div>

              <div className="flex flex-col">
                <FormField
                  control={form.control}
                  name={`reminders.${index}.threshold`}
                  render={() => <FormMessage />}
                />
                <FormField
                  control={form.control}
                  name={`reminders.${index}.message`}
                  render={() => <FormMessage />}
                />
              </div>
            </div>
          ))}
        </div>
      ),
    },
    {
      id: 5,
      title: (
        <Button type="submit" className="-mt-1">
          Submit
        </Button>
      ),
      content:
        ((triggerInfo.alert ||
          triggerInfo.reminders.some((r) => r.trigger)) && (
          <Alert className="mt-2">
            <TriangleAlertIcon className="h-4 w-4" />
            <AlertTitle>Warning</AlertTitle>
            <AlertDescription>
              {triggerInfo.alert &&
              triggerInfo.reminders.some((r) => r.trigger) ? (
                <div>
                  This alert and{" "}
                  {triggerInfo.reminders.filter((r) => r.trigger).length}{" "}
                  {triggerInfo.reminders.filter((r) => r.trigger).length === 1
                    ? "reminder"
                    : "reminders"}{" "}
                  will immediately trigger once submitted.
                </div>
              ) : triggerInfo.alert &&
                triggerInfo.reminders.every((r) => !r.trigger) ? (
                <div>This alert will immediately trigger once submitted.</div>
              ) : triggerInfo.reminders.some((r) => r.trigger) ? (
                <div>
                  {triggerInfo.reminders.filter((r) => r.trigger).length}{" "}
                  {triggerInfo.reminders.filter((r) => r.trigger).length === 1
                    ? "reminder"
                    : "reminders"}{" "}
                  will immediately trigger once submitted.
                </div>
              ) : null}

              <FormField
                control={form.control}
                name="ignoreTrigger"
                render={({ field: { value, onChange, ...field } }) => (
                  <FormItem className="mt-4 flex gap-2 items-center">
                    <FormControl>
                      <Checkbox
                        checked={value}
                        onCheckedChange={onChange}
                        {...field}
                      />
                    </FormControl>
                    <FormLabel>Ignore for current period</FormLabel>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </AlertDescription>
          </Alert>
        )) ||
        null,
    },
  ];

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="space-y-4 p-8 m-auto"
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
                {item.content && (
                  <TimelineContent className="mt-2 space-y-4">
                    {item.content}
                  </TimelineContent>
                )}
              </TimelineItem>
            ))}
          </Timeline>

          <div className="flex justify-end mt-4"></div>
        </form>
      </Form>
    </>
  );
}
