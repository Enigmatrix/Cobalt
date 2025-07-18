import time
import pytest
import logging
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from driver import (
    driver_web_state,
    DriverData,
    events,
    RecordedEvents,
    BrowserWindowSnapshotChange,
    WindowFocused,
)

logger = logging.getLogger(__name__)


@pytest.fixture
def browser():
    chrome_options = Options()
    chrome_options.add_argument("--start-maximized")
    chrome_options.add_argument("--enable-features=UiaProvider")
    driver = webdriver.Chrome(options=chrome_options)
    yield driver
    driver.quit()


def test_switch(
    driver_web_state: DriverData, browser: webdriver.Chrome, events: RecordedEvents
):
    events.push(
        BrowserWindowSnapshotChange(url="data:,", title="data:, - Google Chrome")
    )
    url1, url2 = "https://www.google.com/", "https://enigmatrix.me/"

    # Open first tab and navigate to url1
    logger.info(f"Opening first tab: {url1}")
    browser.get(url1)
    events.push(BrowserWindowSnapshotChange(url=url1, title="Google - Google Chrome"))
    # Wait for Google page to load
    WebDriverWait(browser, 10).until(EC.presence_of_element_located((By.NAME, "q")))
    logger.info(f"{url1} page loaded successfully")

    # Wait 3 seconds on url1
    time.sleep(3)

    # Open second tab and navigate to url2
    logger.info(f"Opening second tab: {url2}")
    browser.execute_script(f"window.open('{url2}', '_blank');")
    browser.switch_to.window(browser.window_handles[1])
    events.push(
        BrowserWindowSnapshotChange(url="about:blank", title="Untitled - Google Chrome")
    )
    events.push(
        BrowserWindowSnapshotChange(url=url2, title="Enigmatrix - Google Chrome")
    )
    # Wait for url2 page to load
    WebDriverWait(browser, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    logger.info(f"{url2} page loaded successfully")
    logger.info(f"Currently active tab: {url2}")

    # Wait 5 seconds on url2
    logger.info(f"Waiting 5 seconds on {url2}...")
    time.sleep(5)

    # Switch to the first tab (url1)
    logger.info(f"Switching to {url1} tab...")
    browser.switch_to.window(browser.window_handles[0])
    events.push(
        BrowserWindowSnapshotChange(
            url=url1, title="Google - Google Chrome", is_incognito=False
        )
    )
    logger.info(f"Currently active tab: {url1}")

    # Wait 5 seconds on url1
    logger.info(f"Waiting 5 seconds on {url1}...")
    time.sleep(5)

    out_events = driver_web_state.events()
    for event in out_events:
        logger.info(f"Event: {event}")

    assert events == out_events

    logger.info(f"Test completed successfully! {events.events} == {out_events}")
    assert False
