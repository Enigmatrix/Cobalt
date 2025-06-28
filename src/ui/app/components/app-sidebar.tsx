import { Button } from "@/components/ui/button";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import {
  ArrowLeftIcon,
  Bell,
  FlaskConical,
  History,
  Home,
  List,
  Settings,
  Tag,
} from "lucide-react";
import * as React from "react";
import { NavLink, useLocation, useNavigate } from "react-router";

const data = {
  footer: [
    ...(import.meta.env.DEV
      ? [
          {
            title: "Experiments",
            url: "/experiments",
            icon: <FlaskConical />,
          },
        ]
      : []),
    {
      title: "Settings",
      url: "/settings",
      icon: <Settings />,
    },
  ],
  navMain: [
    {
      title: "Home",
      url: "/",
      icon: <Home />,
    },
    {
      title: "Apps",
      url: "/apps",
      icon: <List />,
    },
    {
      title: "Tags",
      url: "/tags",
      icon: <Tag />,
    },
    {
      title: "Alerts",
      url: "/alerts",
      icon: <Bell />,
    },
    {
      title: "History",
      url: "/history",
      icon: <History />,
    },
  ],
};

function AppSidebarItem({ title, url, icon }: (typeof data.navMain)[0]) {
  const { pathname } = useLocation();
  const isActive = url === pathname;
  return (
    <SidebarMenuItem key={title}>
      <SidebarMenuButton asChild isActive={isActive} className="px-0">
        <NavLink
          to={url}
          className="text-lg inline-flex items-center rounded-none"
        >
          {/* Active indicator */}
          <div className="h-4 w-1 ml-1">
            {isActive ? (
              <div className="bg-primary h-full rounded-sm w-1"></div>
            ) : null}
          </div>
          {icon}
          {title}
        </NavLink>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}

interface ReactRouterState {
  idx: number;
  // ...
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const navigate = useNavigate();
  const state = window.history.state as ReactRouterState | null;
  const canGoBack = (state && state.idx > 0) ?? false;

  return (
    <Sidebar {...props}>
      <SidebarHeader>
        <div className="flex items-center">
          <Button
            variant="ghost"
            size="icon"
            disabled={!canGoBack}
            onClick={() => navigate(-1)}
          >
            <ArrowLeftIcon />
          </Button>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroupContent>
          <SidebarMenu>
            {data.navMain.map((item) => (
              <AppSidebarItem key={item.title} {...item} />
            ))}
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarContent>

      <SidebarFooter>
        <SidebarContent>
          <SidebarGroupContent>
            <SidebarMenu>
              {data.footer.map((item) => (
                <AppSidebarItem key={item.title} {...item} />
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarContent>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
