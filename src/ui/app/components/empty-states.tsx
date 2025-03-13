import React from "react";
import { cn } from "@/lib/utils";
import { Tag, AppWindow, BellRing, SearchX, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

type EmptyStateType =
  | "tags"
  | "apps"
  | "alerts"
  | "searchTags"
  | "searchApps"
  | "searchAlerts";

type EmptyStateVariant = "small" | "default" | "large";
type ButtonSize = "sm" | "default" | "lg";

interface EmptyStateProps {
  type: EmptyStateType;
  variant?: EmptyStateVariant;
  searchTerm?: string;
  className?: string;
  onCreateClick?: () => void;
}

export function EmptyState({
  type,
  variant = "default",
  searchTerm = "",
  className,
  onCreateClick,
}: EmptyStateProps) {
  // Configuration for each variant
  const variantConfig = {
    small: {
      containerClass: "gap-2 py-3",
      iconSize: 16,
      titleClass: "text-sm",
      descriptionClass: "text-xs",
      iconContainerClass: "p-2",
      buttonSize: "sm" as ButtonSize,
    },
    default: {
      containerClass: "gap-4 py-6",
      iconSize: 24,
      titleClass: "text-lg",
      descriptionClass: "text-sm",
      iconContainerClass: "p-3",
      buttonSize: "default" as ButtonSize,
    },
    large: {
      containerClass: "gap-6 py-10",
      iconSize: 36,
      titleClass: "text-2xl",
      descriptionClass: "text-base",
      iconContainerClass: "p-4",
      buttonSize: "lg" as ButtonSize,
    },
  };

  const config = variantConfig[variant];

  // Content configurations
  const stateConfigs: Record<
    EmptyStateType,
    {
      icon: React.ElementType;
      title: string;
      description: string;
      showCreateButton: boolean;
      createButtonText?: string;
    }
  > = {
    tags: {
      icon: Tag,
      title: "No tags created",
      description: "Create your first tag to organizer your apps.",
      showCreateButton: true,
      createButtonText: "Create Tag",
    },
    apps: {
      icon: AppWindow,
      title: "No apps available",
      description: "Your apps will appear here once tracked.",
      showCreateButton: false,
    },
    alerts: {
      icon: BellRing,
      title: "No alerts created",
      description: "Create an alert to control your usage.",
      showCreateButton: true,
      createButtonText: "Create Alert",
    },
    searchTags: {
      icon: SearchX,
      title: "No tags found",
      description: searchTerm
        ? `No tags match "${searchTerm}".`
        : "No tags found for your search.",
      showCreateButton: false,
    },
    searchApps: {
      icon: SearchX,
      title: "No apps found",
      description: searchTerm
        ? `No apps match "${searchTerm}".`
        : "No apps found for your search.",
      showCreateButton: false,
    },
    searchAlerts: {
      icon: SearchX,
      title: "No alerts found",
      description: searchTerm
        ? `No alerts match "${searchTerm}".`
        : "No alerts found for your search.",
      showCreateButton: false,
    },
  };

  const stateConfig = stateConfigs[type];
  const IconComponent = stateConfig.icon;

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center text-center px-6 py-8 rounded-lg bg-background/50 transition-all duration-300 animate-in fade-in",
        config.containerClass,
        className,
      )}
    >
      <div
        className={cn(
          "rounded-full bg-muted flex items-center justify-center transition-transform duration-500 hover:scale-110",
          config.iconContainerClass,
        )}
      >
        <IconComponent
          size={config.iconSize}
          className="text-muted-foreground animate-[float_3s_ease-in-out_infinite]"
        />
      </div>
      <div className="space-y-1 max-w-[18rem]">
        <h3
          className={cn(
            "font-semibold text-foreground animate-in slide-in-from-bottom-2 duration-300",
            config.titleClass,
          )}
        >
          {stateConfig.title}
        </h3>
        <p
          className={cn(
            "text-muted-foreground animate-in slide-in-from-bottom-3 duration-300",
            config.descriptionClass,
          )}
        >
          {stateConfig.description}
        </p>
      </div>

      {stateConfig.showCreateButton && onCreateClick && (
        <Button
          onClick={onCreateClick}
          size={config.buttonSize}
          className="mt-4 animate-in slide-in-from-bottom-4 duration-300"
        >
          <Plus className="mr-2" size={16} />
          {stateConfig.createButtonText}
        </Button>
      )}
    </div>
  );
}

// For convenience, also export specific empty state components
export const NoTags = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="tags" {...props} />
);
export const NoApps = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="apps" {...props} />
);
export const NoAlerts = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="alerts" {...props} />
);
export const NoTagsFound = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="searchTags" {...props} />
);
export const NoAppsFound = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="searchApps" {...props} />
);
export const NoAlertsFound = (props: Omit<EmptyStateProps, "type">) => (
  <EmptyState type="searchAlerts" {...props} />
);
