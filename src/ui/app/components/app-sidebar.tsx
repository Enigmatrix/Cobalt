import * as React from "react";
import { useNavigate } from "react-router";
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
import { NavLink, useLocation } from "react-router";
import {
  Bell,
  FlaskConical,
  History,
  Home,
  List,
  Settings,
  Tag,
  ActivityIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
} from "lucide-react";

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
      <SidebarMenuButton asChild isActive={isActive} className="py-2 px-0">
        <NavLink to={url} className="text-lg inline-flex items-center">
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

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const navigate = useNavigate();
  const canGoBack = window.history.state && window.history.state.idx > 0;
  const canGoForward =
    window.history.state &&
    window.history.state.idx < window.history.length - 1;

  return (
    <Sidebar {...props}>
      <SidebarHeader>
        <div className="flex items-center">
          <ActivityIcon className="mx-2" size={18} />
          <div className="text-base mr-auto ml-2">Cobalt</div>
          <Button
            variant="ghost"
            size="icon"
            disabled={!canGoBack}
            onClick={() => navigate(-1)}
          >
            <ArrowLeftIcon className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            disabled={!canGoForward}
            onClick={() => navigate(1)}
          >
            <ArrowRightIcon className="h-4 w-4" />
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
