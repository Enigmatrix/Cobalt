import { AlertForm, type FormValues } from "@/components/alert/alert-form";
import { DurationText } from "@/components/time/duration-text";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useZodForm } from "@/hooks/use-form";
import { useAlert } from "@/hooks/use-refresh";
import { useTriggerInfo } from "@/hooks/use-trigger-info";
import type { Alert as AlertEntity, Ref } from "@/lib/entities";
import type { UpdatedAlert } from "@/lib/repo";
import { alertSchema } from "@/lib/schema";
import { useAppState } from "@/lib/state";
import { AppUsageBarChartView, TimeProgressBar } from "@/routes/alerts/create";
import { InfoIcon } from "lucide-react";
import { useCallback } from "react";
import { useFieldArray } from "react-hook-form";
import { useNavigate } from "react-router";
import type { Route } from "../alerts/+types/edit";

export default function EditAlerts({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<AlertEntity>;
  const alert = useAlert(id);
  if (!alert) throw new Error("Alert not found");

  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      ...alert,
      ignoreTrigger: false,
    },
  });
  const updateAlert = useAppState((state) => state.updateAlert);
  const navigate = useNavigate();
  const target = form.watch("target");
  const usageLimit = form.watch("usageLimit");
  const timeFrame = form.watch("timeFrame");
  const reminders = form.watch("reminders");
  const { fields, append, remove, update } = useFieldArray({
    control: form.control,
    name: "reminders",
  });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  const onSubmit = useCallback(
    async (values: FormValues) => {
      // FormValues is not the same as UpdatedAlert
      // the ignoreTrigger in FormValues means ignore all firing alerts and reminders
      // but in UpdatedAlert, it's customizable for each alert and reminder.

      const object: UpdatedAlert = {
        id: alert.id,
        ...structuredClone(values),
        ignoreTrigger: false,
        reminders: values.reminders.map((reminder) => ({
          ...reminder,
          ignoreTrigger: false,
        })),
      };

      if (values.ignoreTrigger) {
        object.ignoreTrigger = triggerInfo.alert;
        object.reminders.forEach((reminder, index) => {
          reminder.ignoreTrigger = triggerInfo.reminders[index].trigger;
        });
      }

      await updateAlert(alert, object);
      await navigate(`/alerts/${alert.id}`);
    },
    [updateAlert, navigate, triggerInfo, alert],
  );

  const handleReminderUpdate = useCallback(
    (index: number, threshold: number) => {
      update(index, { threshold, message: reminders[index].message });
    },
    [update, reminders],
  );

  return (
    <>
      <main className="grid grid-cols-[360px_minmax(0,1fr)] h-full ">
        <div className="max-h-screen overflow-y-auto my-auto">
          <AlertForm
            onSubmit={onSubmit}
            form={form}
            triggerInfo={triggerInfo}
            remindersFields={fields}
            remindersAppend={append}
            remindersRemove={remove}
          />
        </div>
        <div className="flex flex-col p-8">
          <Tabs defaultValue="usage" className="flex-1 flex flex-col">
            <TabsList className="self-center">
              <TabsTrigger value="usage">Usage</TabsTrigger>
              <TabsTrigger value="actions">Actions</TabsTrigger>
              <TabsTrigger value="reminders">Reminders</TabsTrigger>
            </TabsList>
            <TabsContent value="usage" className="flex-1 flex flex-col">
              <AppUsageBarChartView
                target={target}
                usageLimit={usageLimit}
                timeFrame={timeFrame}
              />
            </TabsContent>
            <TabsContent value="actions">
              <div>TODO show action video</div>
            </TabsContent>
            <TabsContent value="reminders">
              <div className="flex h-full">
                {!usageLimit || !timeFrame || !target ? (
                  <Alert className="m-auto">
                    <InfoIcon className="size-4" />
                    <AlertTitle>Choose options first</AlertTitle>
                    <AlertDescription>
                      Select target, period and usage limit to show reminder and
                      usage progress.
                    </AlertDescription>
                  </Alert>
                ) : (
                  <div className="flex-1 flex flex-col gap-2 my-auto">
                    <div className="grid grid-cols-2 gap-4 mb-1">
                      <div className="flex flex-col">
                        <span className="text-sm font-medium text-muted-foreground">
                          Current Usage
                        </span>
                        <div className="flex items-baseline gap-2">
                          <DurationText
                            className="text-lg font-semibold pr-1"
                            ticks={triggerInfo.currentUsage}
                          />
                          {triggerInfo.currentUsage !== 0 && (
                            <span
                              className={`text-sm tabular-nums ${
                                triggerInfo.currentUsage / usageLimit >= 1
                                  ? "text-destructive"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {Math.min(
                                100,
                                (triggerInfo.currentUsage / usageLimit) * 100,
                              ).toFixed(0)}
                              %
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="flex flex-col items-end">
                        <span className="text-sm font-medium text-muted-foreground">
                          Usage Limit
                        </span>
                        <DurationText
                          className="text-lg font-semibold pl-1"
                          ticks={usageLimit}
                        />
                      </div>
                    </div>
                    <TimeProgressBar
                      usageLimit={usageLimit}
                      currentUsage={triggerInfo.currentUsage}
                      reminders={reminders}
                      circleRadius={12}
                      onReminderAdd={(v) => append({ ...v })}
                      onReminderUpdate={handleReminderUpdate}
                    />
                  </div>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </main>
    </>
  );
}
