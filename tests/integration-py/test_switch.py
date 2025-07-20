import time
import pytest
import logging
import itertools
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
import pyautogui
from driver import (
    driver_web_state,
    DriverData,
    events,
    RecordedEvents,
    Change,
    WindowFocused,
)
from server import server, Webserver

logger = logging.getLogger(__name__)


@pytest.fixture
def browser():
    chrome_options = Options()
    # chrome_options.add_argument("--start-maximized")

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
    "chrome_internal": {
        "url": "chrome://flags/1",  # use /1 due to omnibox and document value differences
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

DELAY = 5


@pytest.mark.parametrize("url1,url2", url_pairs)
def test_switch(
    url1: str,
    url2: str,
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    events: RecordedEvents,
):
    url1, title1 = urls[url1]["url"], urls[url1]["title"]
    url2, title2 = urls[url2]["url"], urls[url2]["title"]

    # open url1 and url2 in new tabs
    logger.info(f"Opening first tab: {url1}")
    browser.get(url1)
    logger.info(f"Opening second tab: {url2}")
    pyautogui.hotkey("ctrl", "t")
    pyautogui.typewrite(url2, interval=0.01)
    time.sleep(0.1)
    pyautogui.press("enter")
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    logger.info(f"Switching to back first tab: {url1}")
    browser.switch_to.window(browser.window_handles[0])

    # start driver_web_state after opening tabs
    driver_web_state.start()
    # initial foreground state is url1
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    # Wait for 5 seconds on url1
    logger.info(f"Waiting {DELAY} seconds on {url1}...")
    time.sleep(DELAY)

    # Switch to the first tab (url2)
    logger.info(f"Switching to {url2} tab...")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url2, title=f"{title2} - Google Chrome"))

    # Wait 5 seconds on url2
    logger.info(f"Waiting {DELAY} seconds on {url2}...")
    time.sleep(DELAY)

    # Switch to the first tab (url1)
    logger.info(f"Switching to {url1} tab...")
    browser.switch_to.window(browser.window_handles[0])
    events.push(Change(url=url1, title=f"{title1} - Google Chrome"))

    # Wait 5 seconds on url1
    logger.info(f"Waiting {DELAY} seconds on {url1}...")
    time.sleep(DELAY)

    out_events = driver_web_state.events()
    assert events == out_events


def test_switch_diff_url_same_title(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    server: Webserver,
    events: RecordedEvents,
):
    url1 = f"http://localhost:{server.port}/diff_url_same_title_1.html"
    url2 = f"http://localhost:{server.port}/diff_url_same_title_2.html"
    title = "Same Title"

    # open url1 and url2 in new tabs
    logger.info(f"Opening first tab:")
    browser.get(url1)

    logger.info(f"Opening second tab:")
    browser.switch_to.new_window("tab")
    browser.get(url2)

    logger.info(f"Switching to back first tab:")
    browser.switch_to.window(browser.window_handles[0])

    # start driver_web_state after opening tabs
    driver_web_state.start()
    # initial foreground state is url1
    events.push(Change(url=url1, title=f"{title} - Google Chrome"))

    # Wait for 5 seconds on url1
    logger.info(f"Waiting {DELAY} seconds on {url1}...")
    time.sleep(DELAY)

    # Switch to the first tab (url2)
    logger.info(f"Switching to second tab...")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url2, title=f"{title} - Google Chrome"))

    # Wait 5 seconds on url2
    logger.info(f"Waiting {DELAY} seconds on second tab...")
    time.sleep(DELAY)

    # Switch to the first tab (url1)
    logger.info(f"Switching to first tab...")
    browser.switch_to.window(browser.window_handles[0])
    events.push(Change(url=url1, title=f"{title} - Google Chrome"))

    # Wait 5 seconds on url1
    logger.info(f"Waiting {DELAY} seconds on {url1}...")
    time.sleep(DELAY)

    out_events = driver_web_state.events()
    assert events == out_events


def test_switch_same_url_diff_title(
    driver_web_state: DriverData,
    browser: webdriver.Chrome,
    server: Webserver,
    events: RecordedEvents,
):
    url = f"http://localhost:{server.port}/same_url_diff_title.html"

    # open url1 and url2 in new tabs
    logger.info(f"Opening first tab:")
    browser.get(url)
    title1 = browser.title

    logger.info(f"Opening second tab:")
    browser.switch_to.new_window("tab")
    browser.get(url)
    title2 = browser.title

    logger.info(f"Switching to back first tab:")
    browser.switch_to.window(browser.window_handles[0])
    assert title1 != title2

    # start driver_web_state after opening tabs
    driver_web_state.start()
    # initial foreground state is url
    events.push(Change(url=url, title=f"{title1} - Google Chrome"))

    # Wait for 5 seconds on url
    logger.info(f"Waiting {DELAY} seconds on {url}...")
    time.sleep(DELAY)

    # Switch to the first tab (url)
    logger.info(f"Switching to second tab...")
    browser.switch_to.window(browser.window_handles[1])
    events.push(Change(url=url, title=f"{title2} - Google Chrome"))

    # Wait 5 seconds on url
    logger.info(f"Waiting {DELAY} seconds on second tab...")
    time.sleep(DELAY)

    # Switch to the first tab (url)
    logger.info(f"Switching to first tab...")
    browser.switch_to.window(browser.window_handles[0])
    events.push(Change(url=url, title=f"{title1} - Google Chrome"))

    # Wait 5 seconds on url
    logger.info(f"Waiting {DELAY} seconds on {url}...")
    time.sleep(DELAY)

    out_events = driver_web_state.events()
    assert events == out_events
