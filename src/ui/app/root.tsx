import { AppSidebar } from "@/components/app-sidebar";
import { ThemeProvider } from "@/components/theme-provider";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import { error as errorLog, info } from "@/lib/log";
import { initState } from "@/lib/state";
import { SplashScreen } from "@/splashscreen";
import { openUrl as open } from "@tauri-apps/plugin-opener";
import { Suspense, useEffect, useMemo } from "react";
import {
  Await,
  isRouteErrorResponse,
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLocation,
} from "react-router";
import type { Route } from "./+types/root";
import stylesheet from "./app.css?url";

export const links: Route.LinksFunction = () => [
  { rel: "stylesheet", href: stylesheet },
];

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  const initStatePromise = useMemo(initState, []);
  const location = useLocation();

  useEffect(() => {
    info("navigate:", location);
  }, [location]);

  return (
    <ThemeProvider>
      <Suspense fallback={<SplashScreen />}>
        <Await resolve={initStatePromise}>
          <SidebarProvider>
            <AppSidebar />
            <SidebarInset>
              <Outlet />
            </SidebarInset>
          </SidebarProvider>
        </Await>
      </Suspense>
      <Toaster />
    </ThemeProvider>
  );
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  errorLog("caught at ErrorBoundary", {
    error,
    str: error?.toString(),
    message: (error as Record<string, unknown>)?.message,
    stack: (error as Record<string, unknown>)?.stack,
  });

  const isDev = import.meta.env.DEV;
  let message = "An unexpected error occurred.";
  let stack: string | undefined;

  if (isRouteErrorResponse(error)) {
    message =
      error.status === 404 ? "404 Not Found" : error.statusText || message;
  } else if (error instanceof Error) {
    message = error.message;
    stack = error.stack;
  }

  return (
    <ThemeProvider>
      <main className="pt-16 p-4 container mx-auto">
        <div className="border border-destructive rounded-md p-4 space-y-2 bg-destructive/10">
          <h1 className="text-xl font-bold text-destructive">Error</h1>
          {isDev ? (
            <>
              <p>{message}</p>
              {stack && (
                <pre className="overflow-x-auto text-sm p-2 bg-destructive/5 rounded-md">
                  <code>{stack}</code>
                </pre>
              )}
              Raw Error:
              <pre className="overflow-x-auto text-sm p-2 bg-destructive/5 rounded-md">
                <code>{JSON.stringify(error, null, 2)}</code>
              </pre>
            </>
          ) : (
            <>
              <p>{message}</p>
              <button
                className="px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/90"
                onClick={async () =>
                  await open("https://github.com/Enigmatrix/Cobalt/issues")
                }
              >
                Report Issue
              </button>
            </>
          )}
        </div>
      </main>
    </ThemeProvider>
  );
}
