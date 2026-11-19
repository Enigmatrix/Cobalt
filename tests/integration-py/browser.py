import pytest
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

INCOGNITO_MODE = "incognito"
NORMAL_MODE = "normal"


@pytest.fixture
def browser(request):
    chrome_options = Options()

    # this is enabled by default in Chrome but selenium doesn't enable it by default
    chrome_options.add_argument("--enable-features=UiaProvider")
    chrome_options.add_argument("--log-level=3")
    chrome_options.add_experimental_option("excludeSwitches", ["enable-logging"])
    # chrome 154+ enables https-first mode by default, which shows a warning dialog instead of loading http sites
    chrome_options.add_experimental_option(
        "prefs",
        {"https_only_mode_enabled": False, "https_first_balanced_mode_enabled": False},
    )

    mode = getattr(request, "param", NORMAL_MODE)
    if mode == INCOGNITO_MODE:
        chrome_options.add_argument("--incognito")

    driver = webdriver.Chrome(options=chrome_options)
    yield driver
    driver.quit()
