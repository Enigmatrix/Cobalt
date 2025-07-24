import time
import logging
from selenium import webdriver
from driver import (
    DriverData,
    RecordedEvents,
)
from database import Database, db_time
import pyautogui
import win32gui

logger = logging.getLogger(__name__)

# TODO: shift this somewhere else
DELAY = 5

urls = {
    "https1": {
        "url": "https://youtube.com/watch?v=dQw4w9WgXcQ",
        "base_url": "https://youtube.com",
    },
    "https2": {
        "url": "https://github.com/chromium/chromium",
        "base_url": "https://github.com",
    },
    "https3": {
        "url": "https://docs.rs/backoff/latest/backoff/",
        "base_url": "https://docs.rs",
    },
}


def test_alert_dim_between_tabs(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
):
    url1 = urls["https1"]["url"]
    url2 = urls["https2"]["url"]
    url3 = urls["https3"]["url"]

    # open url1 and url2 in new tabs
    logger.info(f"Opening tab 1: {url1}")
    browser.get(url1)

    logger.info(f"Opening tab 2: {url2}")
    browser.switch_to.new_window("tab")
    browser.get(url2)
    # remove focus from the omnibox
    pyautogui.hotkey("esc")

    logger.info(f"Opening tab 3: {url3}")
    browser.switch_to.new_window("tab")
    browser.get(url3)
    # remove focus from the omnibox
    pyautogui.hotkey("esc")

    logger.info(f"Switching back to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    logger.info("Starting driver_web_state")
    driver_web_state.start()
    logger.info("Waiting for driver_web_state to start")
    time.sleep(5)
    logger.info("Checking if db exists")
    db_path = driver_web_state.save_dir / "main.db"
    assert db_path.exists()
    db = Database(db_path)

    now = db_time()

    # --- Create apps for each URL ---
    app_ids = {}
    for key, info in urls.items():
        app_ids[key] = db.create_app(
            name=info["base_url"],
            description=f"Test app for {info['base_url']}",
            company="TestCo",
            color="#123456",
            icon=None,
            identity_tag=2,  # Website
            identity_text0=info["base_url"],
            tag_id=None,
            created_at=now,
            updated_at=now,
        )

    # --- Create alerts for url2 and url3 ---
    # time_frame=2 (monthly), trigger_action_tag=1 (dim), usage_limit=1s, duration=3h/5h in 100ns ticks
    SECOND_100NS = 10**7
    HOUR_100NS = 60 * 60 * SECOND_100NS
    alert_ids = {}
    alert_ids["https2"] = db.create_alert(
        app_id=app_ids["https2"],
        tag_id=None,
        usage_limit=1 * SECOND_100NS,
        time_frame=2,  # monthly
        trigger_action_tag=1,  # dim
        trigger_action_dim_duration=3 * HOUR_100NS,
        trigger_action_message_content=None,
        active=True,
        created_at=now,
        updated_at=now,
    )
    alert_ids["https3"] = db.create_alert(
        app_id=app_ids["https3"],
        tag_id=None,
        usage_limit=1 * SECOND_100NS,
        time_frame=2,  # monthly
        trigger_action_tag=1,  # dim
        trigger_action_dim_duration=5 * HOUR_100NS,
        trigger_action_message_content=None,
        active=True,
        created_at=now,
        updated_at=now,
    )

    # --- Create sessions for each app ---
    session_ids = {}
    for key, info in urls.items():
        session_ids[key] = db.create_session(
            app_id=app_ids[key],
            title=info["base_url"],
            url=info["url"],
        )

    # --- Create usage for each session (2s duration, at 1 hour and 2 seconds before now) ---
    usage_start = now - (HOUR_100NS + 2 * SECOND_100NS)  # 1 hour and 2 seconds ago
    usage_end = usage_start + 2 * SECOND_100NS  # 2 seconds duration
    for key in urls.keys():
        db.create_usage(
            session_id=session_ids[key],
            start=usage_start,
            end=usage_end,
        )

    # --- Create alert events for each alert (at 1 hour before now, reason=0 for hit) ---
    alert_event_time = now - HOUR_100NS  # 1 hour ago
    for key in ("https2", "https3"):
        print("alert_id=", alert_ids[key])
        print("timestamp=", alert_event_time, now, HOUR_100NS)
        print("reason=", 0)
        db.create_alert_event(
            alert_id=alert_ids[key],
            timestamp=alert_event_time,
            reason=0,  # hit
        )

    logger.info(f"Waiting {DELAY} seconds on tab 1: {url1}")
    time.sleep(DELAY)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])

    # can't do this earlier, because the window hasn't been dimmed so
    # SetLayeredWindowAttributes (and SetWindowLong(GWL_EXSTYLE)) isn't called
    time.sleep(3)
    hwnd = get_foreground_window()
    check_dim_level(hwnd, 1.0 / 3)

    logger.info(f"Waiting {DELAY} seconds on tab 2: {url2}")
    time.sleep(DELAY)
    check_dim_level(hwnd, 1.0 / 3)

    logger.info(f"Switching to tab 3: {url3}")
    browser.switch_to.window(browser.window_handles[2])

    time.sleep(3)
    check_dim_level(hwnd, 1.0 / 5)

    logger.info(f"Waiting {DELAY} seconds on tab 3: {url3}")
    time.sleep(DELAY)
    check_dim_level(hwnd, 1.0 / 5)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])

    time.sleep(0.15)
    check_dim_level(hwnd, 1.0 / 3)

    logger.info(f"Waiting {DELAY} seconds on tab 2: {url2}")
    time.sleep(DELAY)
    check_dim_level(hwnd, 1.0 / 3)

    logger.info(f"Switching to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    time.sleep(0.15)
    check_dim_level(hwnd, 0)

    logger.info(f"Waiting {DELAY} seconds on tab 1: {url1}")
    time.sleep(DELAY)
    check_dim_level(hwnd, 0)

    logger.info(f"Switching to tab 3: {url3}")
    browser.switch_to.window(browser.window_handles[2])

    time.sleep(0.15)
    check_dim_level(hwnd, 1.0 / 5)

    logger.info(f"Waiting {DELAY} seconds on tab 3: {url3}")
    time.sleep(DELAY)
    check_dim_level(hwnd, 1.0 / 5)


def get_dim_level(hwnd: int) -> float:
    (_, bAlpha, _) = win32gui.GetLayeredWindowAttributes(hwnd)
    return bAlpha / 255.0


def get_foreground_window() -> int:
    return win32gui.GetForegroundWindow()


def check_dim_level(hwnd: int, expected_dim_level: float):
    MIN_DIM_LEVEL = 0.5
    TOLERANCE = 0.05
    dim_level = get_dim_level(hwnd)
    assert (1 - dim_level) >= expected_dim_level * (1 - MIN_DIM_LEVEL) - TOLERANCE
    assert (1 - dim_level) <= expected_dim_level * (1 - MIN_DIM_LEVEL) + TOLERANCE
