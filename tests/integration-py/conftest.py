import pytest
import subprocess


@pytest.fixture(scope="session")
def build_driver_web_state():
    subprocess.run(
        ["cargo", "build", "--locked", "--bin", "driver_web_state"], check=True
    )
