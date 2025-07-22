import pytest
import subprocess
from driver import driver_web_state, events  # noqa: F401
from server import server  # noqa: F401


@pytest.fixture(scope="session")
def build_driver_web_state():
    subprocess.run(
        ["cargo", "build", "--locked", "--bin", "driver_web_state"], check=True
    )
