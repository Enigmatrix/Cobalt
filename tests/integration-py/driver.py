import pytest
import logging
import tempfile
import shutil
import subprocess
import json
from pathlib import Path

logger = logging.getLogger(__name__)


class DriverData:
    def __init__(self, file_path: Path):
        self.file_path = file_path

    def lines(self):
        with open(self.file_path, "r", encoding="utf-8") as f:
            return f.readlines()

    def events(self):
        return [json.loads(line) for line in self.lines() if line.strip()]


@pytest.fixture
def driver_web_state(request, build_driver_web_state):
    """
    Pytest fixture to manage the driver_web_state process and data.json output.
    Args:
        appsettings: None or dict. If None, loads $root/dev/appsettings.Debug.json.
    Yields:
        DriverData object for reading lines/events from data.json.
    """
    save_dir = Path(tempfile.mkdtemp(prefix="driver_web_state_"))

    # Write appsettings.json to save_dir
    appsettings = getattr(request, "param", None)
    root = Path(__file__).resolve().parents[2]
    if appsettings is None:
        # Load default appsettings.json
        with open(root / "dev/appsettings.Debug.json", "r", encoding="utf-8") as f:
            appsettings = json.load(f)
    appsettings_path = Path(save_dir) / "appsettings.json"
    with open(appsettings_path, "w", encoding="utf-8") as f:
        json.dump(appsettings, f)

    # Start driver_web_state in background, save stdout/stderr to files
    driver_path = root / "target" / "debug" / "driver_web_state.exe"
    stdout_path = save_dir / "driver_web_state_stdout.txt"
    stderr_path = save_dir / "driver_web_state_stderr.txt"
    driver_program = subprocess.Popen(
        [str(driver_path)],
        cwd=save_dir,
        stdout=open(stdout_path, "w"),
        stderr=open(stderr_path, "w"),
        text=True,
        bufsize=0,
    )

    data_file = Path(save_dir) / "data.json"
    yield DriverData(data_file)

    # Cleanup
    try:
        driver_program.kill()
    except Exception as e:
        logger.warning(f"Failed to terminate driver_program: {e}")

    # Only show output if test failed
    if hasattr(request.node, "rep_call") and request.node.rep_call.failed:
        logger.info("--- driver_web_state stdout ---")
        out = open(stdout_path, "r").read()
        logger.info(out)
        logger.info("--- driver_web_state stderr ---")
        err = open(stderr_path, "r").read()
        logger.info(err)

    shutil.rmtree(save_dir, ignore_errors=True)
