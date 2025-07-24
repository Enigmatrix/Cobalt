import pytest
from selenium import webdriver
from selenium.webdriver.chrome.options import Options


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
