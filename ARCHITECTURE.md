# High-level Architecture

The application is based on a `engine` that runs on the background collecting data, which streams it to listening clients, such as `Cobalt` and `Cobalt.Tray`. Existing data is retreived from a SQLite database.

The overall architecture is as below

## engine

Hooks into the Windows events to capture changes to the foreground window. Information about the window and its corresponding app is retreived, then details about how long it is used for are also saved to a database. This information is also sent to all listening clients for realtime updates.

### Knowing when the foreground window changes

### Collecting details about window and app

### Processing details

## Cobalt / Cobalt.Tray

### Views

### ViewModel

### Model