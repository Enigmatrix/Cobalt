from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
import win32con
import time
import logging
from selenium import webdriver
from driver import DriverData
from database import Database, db_time
import pyautogui
import win32gui
import pytest
from browser import INCOGNITO_MODE, NORMAL_MODE

from constants import (
    BROWSER_OPEN_DELAY,
    BROWSER_URL_FETCH_DELAY,
    BROWSER_FAST_SWITCH_DELAY,
    TICKS_PER_HOUR,
    MAX_DRIVER_INIT_DELAY,
    MIN_DIM_LEVEL,
    TICKS_PER_SECOND,
    UIA_ACTION_DELAY,
)

logger = logging.getLogger(__name__)

DIM_TOLERANCE = 0.05

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


@pytest.mark.parametrize(
    "browser,browser_mode",
    [(NORMAL_MODE, NORMAL_MODE), (INCOGNITO_MODE, INCOGNITO_MODE)],
    indirect=["browser"],
)
def test_alert_dim_between_tabs(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    browser_mode: bool,
):
    """
    Test that the dim level is correct when switching between tabs.
    We create monthly alerts that dim the browser for 1h/3h and 1h/5h that are immediatly triggered.
    Then we switch between tabs and check that the dim level is correct.
    By default in appsettings.json, we don't track incognito tabs - dim level is full for these.


    We don't use browser.switch_to.new_window() because it doesn't work with incognito tabs.
    """
    url1 = urls["https1"]["url"]
    url2 = urls["https2"]["url"]
    url3 = urls["https3"]["url"]
    incognito = browser_mode == INCOGNITO_MODE

    # open url1 and url2 in new tabs
    logger.info(f"Opening tab 1: {url1}")
    browser.get(url1)

    logger.info(f"Opening tab 2: {url2}")
    pyautogui.hotkey("ctrl", "t")
    time.sleep(UIA_ACTION_DELAY)
    pyautogui.typewrite(url2)
    pyautogui.press("enter")
    time.sleep(UIA_ACTION_DELAY)
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    # remove focus from the omnibox
    pyautogui.hotkey("esc")
    # wait for handle to settle
    while len(browser.window_handles) != 2:
        time.sleep(1)

    logger.info(f"Opening tab 3: {url3}")
    pyautogui.hotkey("ctrl", "t")
    time.sleep(UIA_ACTION_DELAY)
    pyautogui.typewrite(url3)
    pyautogui.press("enter")
    time.sleep(UIA_ACTION_DELAY)
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    # remove focus from the omnibox
    pyautogui.hotkey("esc")

    logger.info(f"Switching back to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    print(browser.window_handles)
    # get browser hwnd
    hwnd = get_foreground_window()

    logger.info("Starting driver_web_state")
    driver_web_state.start()
    logger.info("Waiting for driver_web_state to start")
    time.sleep(MAX_DRIVER_INIT_DELAY)
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
            identity_text1="",
            tag_id=None,
            created_at=now,
            updated_at=now,
        )

    # --- Create alerts for url2 and url3 ---
    # time_frame=2 (monthly), trigger_action_tag=1 (dim), usage_limit=1s, duration=3h/5h in 100ns ticks
    alert_ids = {}
    alert_ids["https2"] = db.create_alert(
        app_id=app_ids["https2"],
        tag_id=None,
        usage_limit=1 * TICKS_PER_SECOND,
        time_frame=2,  # monthly
        trigger_action_tag=1,  # dim
        trigger_action_dim_duration=3 * TICKS_PER_HOUR,
        trigger_action_message_content=None,
        active=True,
        created_at=now,
        updated_at=now,
    )
    alert_ids["https3"] = db.create_alert(
        app_id=app_ids["https3"],
        tag_id=None,
        usage_limit=1 * TICKS_PER_SECOND,
        time_frame=2,  # monthly
        trigger_action_tag=1,  # dim
        trigger_action_dim_duration=5 * TICKS_PER_HOUR,
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
    usage_start = now - (
        TICKS_PER_HOUR + 2 * TICKS_PER_SECOND
    )  # 1 hour and 2 seconds ago
    usage_end = usage_start + 2 * TICKS_PER_SECOND  # 2 seconds duration
    for key in urls.keys():
        db.create_usage(
            session_id=session_ids[key],
            start=usage_start,
            end=usage_end,
        )

    # --- Create alert events for each alert (at 1 hour before now, reason=0 for hit) ---
    alert_event_time = now - TICKS_PER_HOUR  # 1 hour ago
    for key in ("https2", "https3"):
        db.create_alert_event(
            alert_id=alert_ids[key],
            timestamp=alert_event_time,
            reason=0,  # hit
        )

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 1: {url1}")
    time.sleep(BROWSER_OPEN_DELAY)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])

    time.sleep(BROWSER_URL_FETCH_DELAY)
    check_dim_level(hwnd, 1.0 / 3 if not incognito else 0)

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 2: {url2}")
    time.sleep(BROWSER_OPEN_DELAY)
    check_dim_level(hwnd, 1.0 / 3 if not incognito else 0)

    logger.info(f"Switching to tab 3: {url3}")
    browser.switch_to.window(browser.window_handles[2])

    time.sleep(BROWSER_URL_FETCH_DELAY)
    check_dim_level(hwnd, 1.0 / 5 if not incognito else 0)

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 3: {url3}")
    time.sleep(BROWSER_OPEN_DELAY)
    check_dim_level(hwnd, 1.0 / 5 if not incognito else 0)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])

    time.sleep(BROWSER_FAST_SWITCH_DELAY)
    check_dim_level(hwnd, 1.0 / 3 if not incognito else 0)

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 2: {url2}")
    time.sleep(BROWSER_OPEN_DELAY)
    check_dim_level(hwnd, 1.0 / 3 if not incognito else 0)

    logger.info(f"Switching to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    time.sleep(BROWSER_FAST_SWITCH_DELAY)
    check_dim_level(hwnd, 0)

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 1: {url1}")
    time.sleep(BROWSER_OPEN_DELAY)
    check_dim_level(hwnd, 0)

    logger.info(f"Switching to tab 3: {url3}")
    browser.switch_to.window(browser.window_handles[2])

    time.sleep(BROWSER_FAST_SWITCH_DELAY)
    check_dim_level(hwnd, 1.0 / 5 if not incognito else 0)

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 3: {url3}")
    time.sleep(BROWSER_OPEN_DELAY)
    check_dim_level(hwnd, 1.0 / 5 if not incognito else 0)


def get_dim_level(hwnd: int) -> float:
    win32gui.SetWindowLong(hwnd, win32con.GWL_EXSTYLE, win32con.WS_EX_LAYERED)
    (_, bAlpha, _) = win32gui.GetLayeredWindowAttributes(hwnd)
    # bAlpha cannot be 0 - it just means that layering was not done
    if bAlpha == 0:
        return 1.0
    return bAlpha / 255.0


def get_foreground_window() -> int:
    return win32gui.GetForegroundWindow()


def check_dim_level(hwnd: int, expected_dim_level: float):
    """
    Check if the dim level of the window is within the expected range.
    This checks if the value is within a tolerance range of the calculated dim level from DimStatus.
    """
    dim_level = get_dim_level(hwnd)
    assert (1 - dim_level) >= expected_dim_level * (1 - MIN_DIM_LEVEL) - DIM_TOLERANCE
    assert (1 - dim_level) <= expected_dim_level * (1 - MIN_DIM_LEVEL) + DIM_TOLERANCE
