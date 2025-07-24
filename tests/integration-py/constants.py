# All durations are in seconds

BROWSER_OPEN_DELAY = 5
BROWSER_URL_FETCH_DELAY = 3  # URL fetch + sentry hit
BROWSER_FAST_SWITCH_DELAY = 0.15

MAX_DRIVER_INIT_DELAY = 5

UIA_ACTION_DELAY = 1

# from engine/src/desktop.rs
MIN_DIM_LEVEL = 0.5

# Tick constants
TICKS_PER_SECOND = 10_000_000  # Number of 100-nanosecond intervals in one second
TICKS_PER_HOUR = (
    60 * 60 * TICKS_PER_SECOND
)  # Number of 100-nanosecond intervals in one hour
WINDOWS_EPOCH_OFFSET = 621355968000000000  # Offset in ticks between Unix epoch (1970-01-01) and Windows epoch (1601-01-01)
