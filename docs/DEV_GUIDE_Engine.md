# Engine Development Guide

This guide covers the development workflow and architecture of the Cobalt Engine component. The Engine is a Rust-based background process that tracks application usage and responds to alerts and reminders.

## Overview

The Engine's responsibilities are:
- Watching foreground windows, user interactions and system events
- Insert app, session, and usage records into the database when events occur
- Insert user interaction periods and system events into the database
- Coordinate with Resolver to get app info
- Periodically check for alerts and reminders and execute actions

## Project Structure

```
src/
├── engine/              # Main engine code
│   ├── src/
│   │   ├── engine.rs    # Core state machine
│   │   ├── sentry.rs    # Alert manager
│   │   ├── resolver.rs  # App information resolver
│   │   ├── cache.rs     # In-memory cache
│   │   ├── ...
│   └── Cargo.toml
├── platform/            # Platform-specific abstractions
│   ├── src/
│   │   ├── objects/     # Window, Process, Timestamp abstractions
│   │   ├── events/      # Event watchers
│   │   ├── ...
│   │   └── error.rs     # Error handling
│   └── Cargo.toml
├── data/                # Database access layer
│   ├── src/
│   │   ├── db/          # Database connection management and queries
│   │   ├── migrations/  # Schema migrations
│   │   └── entities.rs  # Data models
│   └── Cargo.toml
└── util/                # Utility functions and common dependencies
    └── Cargo.toml
```

## Core Components

### Engine

The Engine is the main state machine that processes events from the platform layer. It's implemented in `src/engine/src/engine.rs`.

Key responsibilities:
- Tracking application sessions and usages
- Recording user interaction periods
- Coordinating with Resolver and Sentry

### Sentry

[Sentry](/src/engine/src/sentry.rs) checks for triggered alerts and reminders and runs their respective actions (kill processes, send toast messages, dim windows, and send reminder toast messages with progress), marking them as completed if necessary in a loop.

Key responsibilities:
- Checking for triggerable alerts and reminders
- Executing configured actions (kill processes, display messages, dim windows)
- Marking alerts as completed when triggered

### Resolver

[Resolver](/src/engine/src/resolver.rs) gathers additional information about applications. It is activated by the `Engine` and runs asynchronously, saving the resolved information to the database, and also runs during engine startup to resolve app info for existing apps that are not initialized, in case of previous Resolver failures.

Key responsibilities:
- Resolving application metadata (name, description, company, icon)
- Storing resolved information in the database

### Cache

[Cache](/src/engine/src/cache.rs) is a simple in-memory cache that stores the resolved information about apps and sessions, along with their associated processes and windows. There is only one instance of this cache around, and it is accessed by the `Engine` and `Resolver`.
It is essentially the current state of the desktop.

Key responsibilities:
- Storing app information
- Tracking active sessions and processes

## Platform Layer

Platform-specific objects are defined in [platform/src/objects/mod.rs](/src/platform/src/objects/mod.rs), like `Window`, `Process` and `Timestamp`. There is liberal use of `unsafe` here, as we are interfacing with the OS and need to handle pointers and memory allocation. 

### Objects

Platform-specific objects like Windows, Processes, and Timestamps are defined in `src/platform/src/objects/mod.rs`.

### Events

Platform-specific event watchers are defined in `src/platform/src/events/mod.rs` like `ForegroundEventWatcher`, `InteractionWatcher`, etc.
These are intended to be polled at a regular interval to check for changes in the system. However, it is possible for it to be event-driven rather than poll-based by using the Win32 API (`SetWindowsHookEx` and `SetWinEventHook`).

### Errors & Buffers

We have our own [Buffer](/src/platform/src/buf.rs) trait that boils down to a `&[T]` slice, including a `WideBuffer` trait for `&[u16]` (which can be converted to a `String`). The common use case would be used via the `buf` method to create a new `SmartBuf<T>`, which can be stack or heap-allocated depending on the size.

Errors are defined in [/src/platform/src/errors.rs](/src/platform/src/error.rs) and are used throughout the platform layer.
There are two types of errors: `Win32Error` and `NtError` (Win32 `GetLastError` and `NTSTATUS` error codes, respectively). We don't need to represent `HRESULT` errors as they are already handled by `windows-rs`. We also have `win32!` macros to handle Win32 operations and `adapt_size!` when we need to repeat operations until the size is correct.

## Data Layer

The Data layer manages database operations and schema migrations.

### Database

The main database connection and query execution is handled in `src/data/src/db/mod.rs`. It uses the `sqlx` crate for database operations,
using SQLite Connection Pooling for connections.

### Data Access Layer
Migrations are handled when the `Database` is initialized, in [data/src/migrations/mod.rs](/src/data/src/migrations/mod.rs), by implementing the `Migration` trait (e.g. `migrationX.rs`) and specifying a `version()` larger than the other migrations. Initialization of `Database` automatically runs migrations that are not yet applied. The `Database` holds
the connection and runs the queries, and is able to create new transactions.

`Database` is used by specific objects like `UsageInserter`, `AppUpdater`, `AlertManager` and `Repository`. [Repository](/src/data/src/db/repo.rs) holds more
complicated queries used by the UI, and all output returned by the function must be `Serializable` to that it can be sent to the UI.

## Threading Model

The Engine mainly uses two threads:

1. **Main Thread**: Runs the Engine, Sentry, and Resolver using tokio single-threaded runtime for async operations.
2. **Win32 Event Loop**: A dedicated thread for handling platform-specific events and timers.

## Development
See [DEV_GUIDE.md](DEV_GUIDE.md) for Development Commands.

### Debugging Tools

1. **Visual Studio Code**: Use the Rust extension with launch configurations for debugging.
1. **DataGrip**: For looking at query plans, debugging SQL and analyzing performance.

## Considerations

1. **Win32 API Calls**: Minimize frequent API calls, especially those that might be expensive.
1. **Memory Usage**: Monitor memory usage, especially for the in-memory cache - it's usually around 5MB, max 10MB.
1. **CPU Usage**: Ensure polling intervals are balanced for responsiveness vs. CPU load.
1. **Security**: Use `unsafe` only when necessary, and ensure proper error handling.
1. **Fault Tolerance**: The Engine is designed to be fault-tolerant, but it's not immune to all errors. Catch and retry fallible operations or provide defaults (especially the Win32 API operations), or crash gracefully.
