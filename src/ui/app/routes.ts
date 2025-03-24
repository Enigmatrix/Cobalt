import {
  type RouteConfig,
  index,
  prefix,
  route,
} from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  ...prefix("apps", [
    index("routes/apps/index.tsx"),
    route(":id", "routes/apps/[id].tsx"),
  ]),
  ...prefix("tags", [
    index("routes/tags/index.tsx"),
    route(":id", "routes/tags/[id].tsx"),
  ]),
  ...prefix("alerts", [
    index("routes/alerts/index.tsx"),
    route("create", "routes/alerts/create.tsx"),
    route(":id", "routes/alerts/[id].tsx"),
    route("edit/:id", "routes/alerts/edit.tsx"),
  ]),
  route("history", "routes/history.tsx"),
  route("settings", "routes/settings.tsx"),
  ...(import.meta.env.DEV
    ? [route("experiments", "routes/experiments.tsx")]
    : []),
] satisfies RouteConfig;
