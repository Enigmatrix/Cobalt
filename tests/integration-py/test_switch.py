import time
from typing import Optional
import pyautogui
import pytest
import logging
import itertools
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from driver import (
    DriverData,
    RecordedEvents,
    Change,
)
from server import Webserver

logger = logging.getLogger(__name__)


@pytest.fixture
def browser():
    chrome_options = Options()

    # this is enabled by default in Chrome but selenium doesn't enable it by default
    chrome_options.add_argument("--enable-features=UiaProvider")
    chrome_options.add_argument("--log-level=3")
    chrome_options.add_experimental_option("excludeSwitches", ["enable-logging"])

    driver = webdriver.Chrome(options=chrome_options)
    yield driver
    driver.quit()


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

DELAY = 3


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
        driver_web_state, browser, events, url1, url2, title1=title, title2=title
    )


def test_switch_same_url_diff_title(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    server: Webserver,
    events: RecordedEvents,
):
    url = f"http://localhost:{server.port}/same_url_diff_title.html"

    do_test_switch(
        driver_web_state, browser, events, url, url, title1=None, title2=None
    )


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

    logger.info(f"Waiting {DELAY} seconds on tab 1: {url1}")
    time.sleep(DELAY)

    logger.info(f"Switching to tab 2: {url2}")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url2, title=f"{title2} - Google Chrome"))

    logger.info(f"Waiting {DELAY} seconds on tab 2: {url2}")
    time.sleep(DELAY)

    logger.info(f"Switching to tab 1: {url1}")
    browser.switch_to.window(browser.window_handles[0])
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    logger.info(f"Waiting {DELAY} seconds on tab 1: {url1}")
    time.sleep(DELAY)

    out_events = driver_web_state.events()
    assert events == out_events
