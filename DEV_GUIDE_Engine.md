# Dev Guide (Engine)

## Data

### Entities
- App: An actual app on the system. Uniquely identified by its AppIdentity, but we use an Id as the Primary Key for performance reasons. Icons are stored as blobs. Notably, when an App is inserted not all fields might be initialized, there will be a slightly delay where the Engine finds the actual details to fill in. But the insertion happens first since we need to insert the session and usage as well.
- Session: To keep track of an app and its window titles. A session can be reset if the app is closed or system shutdown. Titles are not unique.
- Usage: A single, continuous usage of an app during a session. Idling might occur, but that is kept track seperately using interaction periods.
- InteractionPeriod: A period of interaction (mouselicks / keystrokes). Once the user spents time idle, an entry is made that ends when the idle begins.
- Tag: A collection of apps under a common name e.g. Gaming, Productivity
- Alert: Record stating that we track an app or tag and perform an action (Kill/Message(Content)/Dim(Duration)) once the usage limit is hit, per day/ week / month.
- Reminder: Record stating that for a certain threshold from an alert's usage limit, we should display a message warning the user.
- AlertEvent: A log of the Alert firing. Used to prevent multiple firings of the same alert, and to show the user how useful their alert is.
- ReminderEvent: A log of the Reminder firing. Used to prevent multiple firings of the same reminder, and to show the user how useful their reminder is.

Alert and Reminders, when duration-related fields are edited (`target`, `usage_limit`, `time_frame`, Reminder's `threshold`) and there is a corresponding AlertEvent / ReminderEvent, create a new version of the themselves and mark the previous ones as inactive. This is to prevent the same Alert or Reminder from losing their historical AlertEvent and ReminderEvents.

If an AlertEvent exists for an Alert's time frame when the usage limit is 5 minutes, it indicates that we do not need to perform the trigger action (e.g. Message) again in that time frame. However, when the user modifies it to 7 hours, or changes the target to a different app, that AlertEvent should not be tagged to this new version of alert and prevent firing the new version of the Alert. Indeed, what we do is we duplicate the Alert (with the changes to the Alert already recorded), then duplicate the Reminders, but not duplicate the AlertEvents and ReminderEvents. We have an optimization when there are no AlertEvents or ReminderEvents for an Alert at all yet; we can just modify the Alert without incrementing the version.

We apply a similar set of rules for Reminders and ReminderEvents.

### Data Access Layer
We use sqlx+sqlite as the database driver in [data/src/db/mod.rs](src/data/src/db/mod.rs) called `Database`. Migrations are handled in the Engine
itself, in [data/src/migrations/mod.rs](src/data/src/migrations/mod.rs), by implementing the `Migration` trait (e.g. `migrationX.rs`) and specifiying an `version()` larger than the other migrations. Initialization of `Database` automatically runs migrations that are not yet applied. The `Database` holds
the connection and runs the queries, and is able to create new transactions.

`Database` is used by specific objects like `UsageInserter`, `AppUpdater`, `AlertManager` and `Repository`. [Repository](src/data/src/db/repo.rs) holds more
complicated queries used by the UI, and all output returned by the function must be `Serializable` to that it can be sent to the UI.

## Platform

### Errors & Buffers

We have our own [Buffer](./src/platform/src/buf.rs) trait that boils down to a `&[T]` slice, including a `WideBuffer` trait for `&[u16]` (which can be converted to a `String`). The common use case would be used via the `buf` method to create a new `SmartBuf<T>`, which can be stack or heap-allocated depending on the size.

Errors are defined in [platform/src/errors.rs](./src/platform/src/error.rs) and are used throughout the platform layer.
There are two types of errors: `Win32Error` and `NtError` (Win32 `GetLastError` and `NTSTATUS` error codes, respectively). We don't need to represent `HRESULT` errors as they are already handled by `windows-rs`. We also have `win32!` macros to handle Win32 operations and `adapt_size!` when we need to repeat operations until the size is correct.

### Objects

Platform-specific objects are defined in [platform/src/objects/mod.rs](src/platform/src/objects/mod.rs), like `Window`, `Process` and `Timestamp`. There is liberal use of `unsafe` here, as we are interfacing with the OS and need to handle pointers and memory allocation. 

### Events

Platform-specific events are defined in [platform/src/events/mod.rs](src/platform/src/objects/mod.rs), like `InteractionWatcher` and `ForegroundEventWatcher`. These are intended to be polled at a regular interval to check for changes in the system. However, it is possible for it to be event-driven rather than poll-based by using the Win32 API (`SetWindowsHookEx` and `SetWinEventHook`).

## Engine

The Engine and Sentry (as well as Resolver) are run in their own thread using the `tokio` library, asynchronously in their own loops.

We have two threads, the main thread running the above, and the Win32 event loop.

### Event Loop
Within an Win32 event loop, we create Timers to poll the `InteractionWatcher` and `ForegroundEventWatcher` at a regular intervals, and another to send ticks to `Sentry` to rerun its tasks.
This event loop is run in its own thread since `InteractionWatcher`'s code needs to be run as fast as possible.

### Sentry

[Sentry](./src/engine/src/sentry.rs) checks for triggered alerts and reminders and runs their respective actions (kill processes, send toast messages, dim windows, and send reminder toast messages with progress), marking them as completed if necessary in a loop.

### Engine

[Engine](./src/engine/src/engine.rs) is the main driver and runs in a loop, acting as a state machine that moves according to `Event`s from a channel (sent by the Win32 event loop). It inserts usages, sessions and apps into the database, and activates the resolver (spawned asynchronously) to resolve more information about the app. It also records the interaction period and saves it to the database.

### Resolver
[Resolver](./src/engine/src/resolver.rs) runs in a loop and resolves information about apps, such as their name, description, company and logo. It is activated by the `Engine` and runs asynchronously, saving the resolved information to the database.

### Cache
[Cache](./src/engine/src/cache.rs) is a simple in-memory cache that stores the resolved information about apps and sessions, along with their associated processes and windows. There is only one instance of this cache around, and it is accessed by the `Engine` and `Resolver`.

## Util

We use utils to import common libraries. We use `flume` for channels, 
`config-rs` for configuration (read from `appsettings.json`, same as the Viewer), `color-eyre` for errors, `future-rs` for futures and `tracing / tracing-subscriber` for tracing. We also define our own time system (well, its traits atleast) to be implemented by the `platform::objects::Timestamp`.