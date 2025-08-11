# UI Development Guide

This guide covers the development workflow and architecture of the Cobalt UI (Viewer) component. The UI is built as a Tauri application with a React frontend.

## Stack

- [bun](https://bun.sh/)
- [Vite](https://vite.dev/) + [React](https://react.dev/) + [react-router-v7](https://reactrouter.com/home) setup using `bun create vite@latest`
  - `react-router-v7` is configured with `ssr: false` so that the build output is a SPA rather than SSR/SSG - required by Tauri.
- [shadcn](https://ui.shadcn.com/) + [Tailwind CSS](https://tailwindcss.com/) setup using [Vite](https://ui.shadcn.com/docs/installation/vite)
- [ECharts](https://echarts.apache.org/examples/en/index.html) and [Recharts](https://recharts.org/en-US/) for visualizations
- [Tauri](https://v2.tauri.app/) for the desktop app
- [zustand](https://zustand.docs.pmnd.rs/) for state management
- [immer](https://immerjs.github.io/immer/) for state updates
- [eslint](https://eslint.org/) and [prettier](https://prettier.io/) for linting and formatting

## Routes

Routes are setup the via the method in https://reactrouter.com/start/framework/routing in the [/src/ui/app/routes.ts](/src/ui/app/routes.ts) file.

We currently have routes for the following:
- Home: [/src/ui/app/routes/home.tsx](/src/ui/app/routes/home.tsx)
- Apps: [/src/ui/app/routes/apps/index.tsx](/src/ui/app/routes/apps/index.tsx)
    - App: [/src/ui/app/routes/apps/[id].tsx](/src/ui/app/routes/apps/[id].tsx)
- Tags: [/src/ui/app/routes/tags/index.tsx](/src/ui/app/routes/tags/index.tsx)
    - Tag: [/src/ui/app/routes/tags/[id].tsx](/src/ui/app/routes/tags/[id].tsx)
- Alerts: [/src/ui/app/routes/alerts/index.tsx](/src/ui/app/routes/alerts/index.tsx)
    - Alert: [/src/ui/app/routes/alerts/[id].tsx](/src/ui/app/routes/alerts/[id].tsx)
    - Edit Alert: [/src/ui/app/routes/alerts/edit.tsx](/src/ui/app/routes/alerts/edit.tsx)
    - Create Alert: [/src/ui/app/routes/alerts/create.tsx](/src/ui/app/routes/alerts/create.tsx)
- History: [/src/ui/app/routes/history.tsx](/src/ui/app/routes/history.tsx)
- Experiments (only in `dev`): [/src/ui/app/routes/experiments.tsx](/src/ui/app/routes/experiments.tsx)
- Settings: [/src/ui/app/routes/settings.tsx](/src/ui/app/routes/settings.tsx)

## Project Structure

```
src/ui/
├── app/                    # React application code
│   ├── components/         # Reusable UI components
│   │   ├── ui/             # Shadcn components
│   │   ├── viz/            # Visualization components
│   │   └── ...             # Other components
│   ├── hooks/              # Utility functions and hooks
│   ├── lib/                # Utility functions and hooks
│   │   ├── repo.ts         # Wrappers over Tauri repo commands
│   │   ├── schema.ts       # Zod Form Schemas
│   │   ├── entities.ts     # Entity definitions
│   │   ├── time.ts         # Time utilities
│   │   └── state.ts        # Global state management
│   ├── routes/             # Route components for each page
│   └── routes.ts           # Route definitions
├── public/                 # Static assets
├── src-tauri/              # Tauri backend code
│   ├── src/                # Rust source code
│   │   │── bin/engine.rs   # Engine backend - this lets it be embedded in the installer
│   │   │── repo.rs         # Tauri repo commands
│   │   │── state.rs        # Global state management
│   │   │── lib.rs          # Command registrations and API
│   │   └── ...
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── package.json            # Frontend dependencies
├── vite.config.ts          # Vite configuration
├── tsconfig.json           # TypeScript configuration
├── eslint.config.js        # ESLint configuration
├── prettier.config.js      # Prettier configuration
└── ...
```

## Development Workflow

### Local Development

1. Run `bun i`
1. Start the development server: `bun dev`

The development server includes:
- Hot-reloading for React components
- Automatic rebuilding of Rust code when changed
- Source maps for debugging

### Recommended VS Code Extensions

- Tailwind CSS IntelliSense
- ESLint
- Prettier
- rust-analyzer

## UI Architecture

### Components
We put all Shadcn components in [/src/ui/app/components/ui](/src/ui/app/components/ui). Components that are not Shadcn components are put in [/src/ui/app/components](/src/ui/app/components), in appropriate subdirectories e.g. viz, apps, tags, etc.

### State Management

We use `zustand` to manage state in [/src/ui/app/lib/state.ts](/src/ui/app/lib/state.ts). The store can be used via the `useAppState` hook.

The store can be refreshed using `refresh()`, but must be initialized via `initState()` first. Note that this function also initializes the corresponding state on the Tauri Rust side.

### Calling Rust Functions

The Tauri backend exposes functions that can be called from the React frontend. These are defined in `src/ui/src-tauri/src/lib.rs` and can be called directly from JavaScript/TypeScript.
All output must be `Serializable`. Fallibility is handled by the `AppResult` type, and the error is passed to the UI.

You can wrap queries in `useRepo` hooks to get a hook that fetches the data and updates the state during refresh. Example for `getSystemEvents`:

```typescript
// src/ui/app/hooks/use-repo.tsx
export const useSystemEvents = makeUseRepo(getSystemEvents, []);
```

```typescript
// src/ui/app/lib/repo.ts
export async function getSystemEvents({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<SystemEvent[]> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_system_events", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}
```

## Debugging

1. **Large Data Sets**: When dealing with large datasets (e.g., extensive usage history), implement virtualization for lists and tables.

1. **State Updates**: Avoid frequent state updates that trigger re-renders across the application. Use React Scan to figure out which components are causing the issue.

### Debugging Tips

1. Use the Developer Tools in the application (accessible via `F12` in development mode) to inspect network requests, console output, and React component state.

1. Logs are at `./logs/` for dev.

1. Console output is not piped into logs, so use the Developer Tools for that.