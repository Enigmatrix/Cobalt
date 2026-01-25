import {
  AlertFormContainer,
  type FormValues,
} from "@/components/alert/alert-form";
import AppIcon from "@/components/app/app-icon";
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
import { Text } from "@/components/ui/text";
import { useZodForm } from "@/hooks/use-form";
import { useHistoryRef } from "@/hooks/use-history-state";
import { useAlert, useApp, useTag } from "@/hooks/use-refresh";
import { useTriggerInfo } from "@/hooks/use-trigger-info";
import type { Alert as AlertEntity, Ref } from "@/lib/entities";
import type { UpdatedAlert } from "@/lib/repo";
import { alertSchema } from "@/lib/schema";
import { useAppState } from "@/lib/state";
import { TagIcon } from "lucide-react";
import { useCallback, useEffect } from "react";
import { NavLink, useNavigate } from "react-router";
import type { Route } from "../alerts/+types/edit";

export default function EditAlerts({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<AlertEntity>;
  const alert = useAlert(id);
  if (!alert) return null;
  return <EditAlertPage alert={alert} />;
}

function EditAlertPage({ alert }: { alert: AlertEntity }) {
  const defaultFormValues = {
    ...alert,
    ignoreTrigger: false,
  } as unknown as FormValues;
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

  const updateAlert = useAppState((state) => state.updateAlert);
  const navigate = useNavigate();

  // For computing trigger info in onSubmit
  const target = form.watch("target");
  const usageLimit = form.watch("usageLimit");
  const timeFrame = form.watch("timeFrame");
  const reminders = form.watch("reminders");
  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  // Get target entity for breadcrumb
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);
  const targetEntity = app ?? tag;
  const targetName = targetEntity?.name ?? "Unknown";

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

      const newAlertId = await updateAlert(alert, object);
      await navigate(`/alerts/${newAlertId}`, { replace: true });
    },
    [updateAlert, navigate, triggerInfo, alert],
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
              <BreadcrumbLink asChild>
                <NavLink
                  to={`/alerts/${alert.id}`}
                  className="inline-flex items-center overflow-hidden"
                >
                  {app && (
                    <AppIcon app={app} className="w-5 h-5 mr-2 shrink-0" />
                  )}
                  {tag && (
                    <TagIcon
                      className="w-5 h-5 mr-2 shrink-0"
                      style={{ color: tag.color }}
                    />
                  )}
                  <Text>{targetName}</Text>
                </NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem className="overflow-hidden">
              <BreadcrumbPage>Edit</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]">
        <div className="max-w-3xl mx-auto p-4">
          <AlertFormContainer
            onSubmit={onSubmit}
            form={form}
            submitButtonText="Save Changes"
          />
        </div>
      </div>
    </>
  );
}
