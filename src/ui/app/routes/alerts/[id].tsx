import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../alerts/+types/[id]";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
  BreadcrumbLink,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { NavLink, useNavigate } from "react-router";
import { Button, buttonVariants } from "@/components/ui/button";
import { Edit2Icon, TrashIcon } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { useAppState } from "@/lib/state";
import { useCallback } from "react";
import type { Alert, Ref } from "@/lib/entities";
import { useAlert } from "@/hooks/use-refresh";

export default function Page({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<Alert>;
  const alert = useAlert(id);
  if (!alert) return;
  return <AlertPage alert={alert} />;
}

function AlertPage({ alert }: { alert: Alert }) {
  const removeAlert = useAppState((state) => state.removeAlert);
  const navigate = useNavigate();
  const remove = useCallback(async () => {
    await navigate("/alerts");
    await removeAlert(alert.id);
  }, [removeAlert, navigate, alert.id]);

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink asChild>
                <NavLink to="/alerts">Alerts</NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage>Alert ID {alert.id}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* Alert Info */}
          <div className="rounded-xl bg-card border border-border p-6">
            <div className="flex flex-col gap-4">
              {/* Header with name and icon */}
              <div className="flex items-center gap-4">
                <div className="flex-1" />
                <Button variant="outline" size="icon" asChild>
                  <NavLink to={`/alerts/edit/${alert.id}`}>
                    <Edit2Icon />
                  </NavLink>
                </Button>
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button size="icon" variant="outline">
                      <TrashIcon />
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Remove Alert?</AlertDialogTitle>
                      <AlertDialogDescription>
                        This action cannot be undone. All alert history will be
                        removed.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={remove}
                        className={buttonVariants({ variant: "destructive" })}
                      >
                        Remove
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </div>
            </div>
          </div>

          <div className="grid auto-rows-min gap-4 md:grid-cols-3">
            <div className="aspect-video rounded-xl bg-muted/50" />
            <div className="aspect-video rounded-xl bg-muted/50" />
            <div className="aspect-video rounded-xl bg-muted/50" />
          </div>
          <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
        </div>
      </div>
    </>
  );
}
