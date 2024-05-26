# Dev Guide (Engine)

## Data

### Entities
Same as the Viewer defined in [DEV_GUIDE_Viewer](./DEV_GUIDE_Viewer.md).

### Data Access Layer
We use rusqlite as the database driver in [data/src/db/mod.rs](src/Cobalt.Engine/data/src/db/mod.rs) called `Database`. Migrations are handled in the Engine
itself, in [data/src/migrations/mod.rs](src/Cobalt.Engine/data/src/migrations/mod.rs), by implementing the `Migration` trait (e.g. `migrationX.rs`) and specifiying an `version()` larger than the other migrations. Initialization of `Database` automatically runs migrations that are not yet applied.

The `Database` is NOT thread-safe (not `Send`). Also, since Rust does not allow us to hold ownership of an object and a reference and pass both together,
we instead rely on using references to the `Database` object to create specific objects like `UsageInserter`, `AppUpdater` and `AlertManager` that hold
prepared statements and execute them in their methods.

## Platform

### Errors & Buffers

We have our own [Buffer](./src/Cobalt.Engine/platform/src/buf.rs) trait that boils down to a `&[T]` slice, including a `WideBuffer` trait for `&[u16]` (which can be converted to a `String`). The common use case would be used via the `buf` method to create a new `SmartBuf<T>`, which can be stack or heap-allocated depending on the size.

Errors are defined in [platform/src/errors.rs](./src/Cobalt.Engine/platform/src/error.rs) and are used throughout the platform layer.
There are two types of errors: `Win32Error` and `NtError` (Win32 `GetLastError` and `NTSTATUS` error codes, respectively). We don't need to represent `HRESULT` errors as they are already handled by `windows-rs`. We also have `win32!` macros to handle Win32 operations and `adapt_size!` when we need to repeat operations until the size is correct.

### Objects

Platform-specific objects are defined in [platform/src/objects/mod.rs](src/Cobalt.Engine/platform/src/objects/mod.rs), like `Window`, `Process` and `Timestamp`. There is liberal use of `unsafe` here, as we are interfacing with the OS and need to handle pointers and memory allocation. 

### Events

Platform-specific events are defined in [platform/src/events/mod.rs](src/Cobalt.Engine/platform/src/objects/mod.rs), like `InteractionWatcher` and `ForegroundEventWatcher`. These are intended to be polled at a regular interval to check for changes in the system. However, it is possible for it to be event-driven rather than poll-based by using the Win32 API (`SetWindowsHookEx` and `SetWinEventHook`).

## Engine

The Engine and Sentry (as well as Resolver) are run in their own thread using the `futures` library, asynchronously in their own loops.

We have two threads, the main thread running the above, and the Win32 event loop.

### Event Loop
Within an Win32 event loop, we create Timers to poll the `InteractionWatcher` and `ForegroundEventWatcher` at a regular intervals, and another to send ticks to `Sentry` to rerun its tasks.
This event loop is run in its own thread since `InteractionWatcher`'s code needs to be run as fast as possible.

### Sentry

[Sentry](./src/Cobalt.Engine/engine/src/sentry.rs) checks for triggered alerts and reminders and runs their respective actions (kill processes, send toast messages, dim windows, and send reminder toast messages with progress), marking them as completed if necessary in a loop.

### Engine

[Engine](./src/Cobalt.Engine/engine/src/engine.rs) is the main driver and runs in a loop, acting as a state machine that moves according to `Event`s from a channel (sent by the Win32 event loop). It inserts usages, sessions and apps into the database, and activates the resolver (spawned asynchronously) to resolve more information about the app. It also records the interaction period and saves it to the database.

### Resolver
[Resolver](./src/Cobalt.Engine/engine/src/resolver.rs) runs in a loop and resolves information about apps, such as their name, description, company and logo. It is activated by the `Engine` and runs asynchronously, saving the resolved information to the database.

### Cache
[Cache](./src/Cobalt.Engine/engine/src/cache.rs) is a simple in-memory cache that stores the resolved information about apps and sessions, along with their associated processes and windows. There is only one instance of this cache around, and it is accessed by the `Engine` and `Resolver` as a `Rc<RefCell<Cache>>`.

## Util

We use utils to import common libraries. We use `flume` for channels, 
`config-rs` for configuration (read from `appsettings.json`, same as the Viewer), `color-eyre` for errors, `future-rs` for futures and `tracing / tracing-subscriber` for tracing. We also define our own time system (well, its traits atleast) to be implemented by the `platform::objects::Timestamp`.