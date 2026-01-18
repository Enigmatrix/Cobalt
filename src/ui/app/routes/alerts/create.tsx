import {
  AlertFormContainer,
  type FormValues,
} from "@/components/alert/alert-form";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { useZodForm } from "@/hooks/use-form";
import { useHistoryRef } from "@/hooks/use-history-state";
import { useTriggerInfo } from "@/hooks/use-trigger-info";
import type { CreateAlert } from "@/lib/repo";
import { alertSchema } from "@/lib/schema";
import { useAppState } from "@/lib/state";
import { useCallback, useEffect } from "react";
import { NavLink, useNavigate } from "react-router";

const defaultFormValues = {
  ignoreTrigger: false,
  reminders: [],
} as unknown as FormValues;

export default function CreateAlerts() {
  const [savedFormValues, setSavedFormValues] = useHistoryRef<FormValues>(
    defaultFormValues,
    "formState",
  );
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: savedFormValues,
  });

  // Persist form values to history
  const formValues = form.watch();
  useEffect(() => {
    setSavedFormValues(formValues);
  }, [formValues, setSavedFormValues]);

  const createAlert = useAppState((state) => state.createAlert);
  const navigate = useNavigate();

  // For computing trigger info in onSubmit
  const target = form.watch("target");
  const usageLimit = form.watch("usageLimit");
  const timeFrame = form.watch("timeFrame");
  const reminders = form.watch("reminders");
  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  const onSubmit = useCallback(
    async (values: FormValues) => {
      // FormValues is not the same as CreateAlert
      // the ignoreTrigger in FormValues means ignore all firing alerts and reminders
      // but in CreateAlert, it's customizable for each alert and reminder.

      const object: CreateAlert = {
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

      await createAlert(object);
      await navigate("/alerts");
    },
    [createAlert, navigate, triggerInfo],
  );

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap overflow-hidden">
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink asChild>
                <NavLink to="/alerts">Alerts</NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem className="overflow-hidden">
              <BreadcrumbPage>Create Alert</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]">
        <div className="max-w-3xl mx-auto p-4">
          <AlertFormContainer
            onSubmit={onSubmit}
            form={form}
            submitButtonText="Create Alert"
          />
        </div>
      </div>
    </>
  );
}
