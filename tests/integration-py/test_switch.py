import time
import pytest
import logging
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

logger = logging.getLogger(__name__)


@pytest.fixture
def driver():
    chrome_options = Options()
    chrome_options.add_argument("--start-maximized")
    chrome_options.add_argument("--enable-features=UiaProvider")
    driver = webdriver.Chrome(options=chrome_options)
    yield driver
    driver.quit()


def test_switch(driver):
    url1, url2 = "https://www.google.com", "https://enigmatrix.me"

    # Open first tab and navigate to url1
    logger.info(f"Opening first tab: {url1}")
    driver.get(url1)
    # Wait for Google page to load
    WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.NAME, "q")))
    logger.info(f"{url1} page loaded successfully")

    # Open second tab and navigate to url2
    logger.info(f"Opening second tab: {url2}")
    driver.execute_script(f"window.open('{url2}', '_blank');")
    driver.switch_to.window(driver.window_handles[1])
    # Wait for url2 page to load
    WebDriverWait(driver, 10).until(
        EC.presence_of_element_located((By.TAG_NAME, "body"))
    )
    logger.info(f"{url2} page loaded successfully")
    logger.info(f"Currently active tab: {url2}")

    # Wait 5 seconds on url2
    logger.info(f"Waiting 5 seconds on {url2}...")
    time.sleep(5)

    # Switch to the first tab (url1)
    logger.info(f"Switching to {url1} tab...")
    driver.switch_to.window(driver.window_handles[0])
    logger.info(f"Currently active tab: {url1}")

    # Wait 5 seconds on url1
    logger.info(f"Waiting 5 seconds on {url1}...")
    time.sleep(5)

    logger.info("Test completed successfully!")
