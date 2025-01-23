import * as React from "react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { NavLink, useLocation } from "react-router";
import { Bell, History, Home, List, Settings, Tag } from "lucide-react";
import { cn } from "@/lib/utils";

const data = {
  footer: [
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
              <div className="bg-blue-500 h-full rounded-sm w-1"></div>
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
  const { pathname } = useLocation();

  return (
    <Sidebar {...props}>
      <SidebarHeader></SidebarHeader>

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
