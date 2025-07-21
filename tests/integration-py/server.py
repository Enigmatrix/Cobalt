import pytest
from dataclasses import dataclass
import os
import subprocess
import random


@dataclass
class Webserver:
    port: int


@pytest.fixture
def server():
    static_dir = os.path.join(os.path.dirname(__file__), "static")
    port = random.randint(10000, 65535)

    # Start the server as a subprocess
    process = subprocess.Popen(
        ["python", "-m", "http.server", str(port), "--directory", static_dir],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Yield the Webserver instance
    yield Webserver(port=port)

    # Terminate the subprocess
    process.kill()
