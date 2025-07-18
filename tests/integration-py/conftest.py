import pytest
import subprocess


@pytest.fixture(scope="session")
def build_driver_web_state():
    subprocess.run(["cargo", "build", "--bin", "driver_web_state"], check=True)
