import time
from typing import Optional
import pyautogui
import pytest
import logging
import itertools
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from driver import (
    DriverData,
    RecordedEvents,
    Change,
)
from server import Webserver
from constants import BROWSER_OPEN_DELAY

logger = logging.getLogger(__name__)


urls = {
    "https1": {
        "url": "https://chromium.googlesource.com/chromium/src/+/refs/heads/lkgr/docs/design/README.md",
        "title": "Chromium Docs - Chromium Design Docs",
    },
    "https2": {
        "url": "https://github.com/chromium/chromium",
        "title": "GitHub - chromium/chromium: The official GitHub mirror of the Chromium source",
    },
    "http1": {
        "url": "http://textfiles.com/",
        "title": "T E X T F I L E S D O T C O M",
    },
    "chrome_internal": {
        "url": "chrome://flags/",
        "title": "Experiments",
    },
    "data_html": {
        "url": "data:text/html,<html><head><title>nanisore</title></head><body><h1>Hello, World!</h1></body></html>",
        "title": "nanisore",
    },
    "data_png": {
        "url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==",
        "title": "w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg== (5×5)",  # this is a red dot png image
    },
}


url_pairs = itertools.permutations(urls.keys(), 2)


@pytest.mark.parametrize("url1,url2", url_pairs)
def test_switch_various(
    url1: str,
    url2: str,
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
):
    url1, title1 = urls[url1]["url"], urls[url1]["title"]
    url2, title2 = urls[url2]["url"], urls[url2]["title"]

    do_test_switch(driver_web_state, browser, events, url1, url2, title1, title2)


def test_switch_new_window(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
):
    url1, url2 = "https1", "https2"
    url1, title1 = urls[url1]["url"], urls[url1]["title"]
    url2, title2 = urls[url2]["url"], urls[url2]["title"]

    do_test_open_new(
        driver_web_state, browser, events, open_new_window, url1, url2, title1, title2
    )


def test_switch_new_dialog(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
):
    url1, url2 = "https1", "https2"
    url1, title1 = urls[url1]["url"], urls[url1]["title"]
    url2, title2 = urls[url2]["url"], urls[url2]["title"]

    do_test_open_new(
        driver_web_state, browser, events, open_new_dialog, url1, url2, title1, title2
    )


# TODO: test for opening 'Extensions' dialog, 'User' dialog, 'Icon' dialog, 'Ctrl-F' dialog, etc.


def test_switch_diff_url_same_title(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    server: Webserver,
    events: RecordedEvents,
):
    url1 = f"http://localhost:{server.port}/diff_url_same_title_1.html"
    url2 = f"http://localhost:{server.port}/diff_url_same_title_2.html"
    title = "Same Title"

    do_test_switch(
        driver_web_state,
        browser,
        events,
        url1,
        url2,
        title1=title,
        title2=title,
    )


def test_switch_same_url_diff_title(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    server: Webserver,
    events: RecordedEvents,
):
    url = f"http://localhost:{server.port}/same_url_diff_title.html"

    do_test_switch(
        driver_web_state,
        browser,
        events,
        url,
        url,
        title1=None,
        title2=None,
    )


def open_new_window(browser: webdriver.Chrome, url: str):
    browser.switch_to.new_window("window")
    browser.get(url)
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    # remove focus from the omnibox. not really needed here, but it's for consistency
    pyautogui.hotkey("esc")


def open_new_dialog(browser: webdriver.Chrome, url: str):
    browser.execute_script(f"window.open('{url}', '_blank', 'popup');")
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    # don't remove focus from the omnibox - that kills the navigation to the dialog, shows about:blank


def do_test_switch(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
    url1: str,
    url2: str,
    title1: Optional[str] = None,
    title2: Optional[str] = None,
):
    # open url1 and url2 in new tabs
    logger.info(f"Opening tab 1: {url1}")
    browser.get(url1)
    title1 = title1 if title1 is not None else browser.title

    logger.info(f"Opening tab 2: {url2}")
    browser.switch_to.new_window("tab")
    browser.get(url2)
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    # remove focus from the omnibox
    pyautogui.hotkey("esc")

    title2 = title2 if title2 is not None else browser.title

    logger.info(f"Switching back to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    logger.info("Starting driver_web_state")
    driver_web_state.start()
    # initial foreground state is url1
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 1: {url1}")
    time.sleep(BROWSER_OPEN_DELAY)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url2, title=f"{title2} - Google Chrome"))

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 2: {url2}")
    time.sleep(BROWSER_OPEN_DELAY)

    logger.info(f"Switching to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab 1: {url1}")
    time.sleep(BROWSER_OPEN_DELAY)

    out_events = driver_web_state.events()
    assert events == out_events


def do_test_open_new(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
    open_new,
    url1: str,
    url2: str,
    title1: Optional[str] = None,
    title2: Optional[str] = None,
):
    # open url1 and url2 in new tabs
    logger.info(f"Opening tab/window 1: {url1}")
    browser.get(url1)
    title1 = title1 if title1 is not None else browser.title

    logger.info(f"Opening tab/window 2: {url2}")
    open_new(browser, url2)
    title2 = title2 if title2 is not None else browser.title

    logger.info(f"Switching back to tab/window 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    logger.info("Starting driver_web_state")
    driver_web_state.start()
    # initial foreground state is url1
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab/window 1: {url1}")
    time.sleep(BROWSER_OPEN_DELAY)

    logger.info(f"Switching to tab/window 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url2, title=f"{title2} - Google Chrome"))

    logger.info(f"Waiting {BROWSER_OPEN_DELAY} seconds on tab/window 2: {url2}")
    time.sleep(BROWSER_OPEN_DELAY)

    out_events = driver_web_state.events()
    assert events == out_events
