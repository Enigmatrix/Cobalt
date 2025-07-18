import time
import pytest
import logging
import tempfile
import shutil
import subprocess
import json
from pathlib import Path
from dataclasses import dataclass
from typing import Union, Optional, Dict, Any

logger = logging.getLogger(__name__)


class DriverData:
    def __init__(self, file_path: Path):
        self.file_path = file_path

    def lines(self):
        with open(self.file_path, "r", encoding="utf-8") as f:
            return f.readlines()

    def events(self):
        return [
            Event.from_json(json.loads(line)) for line in self.lines() if line.strip()
        ]


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


@dataclass
class Event:
    timestamp: int

    def __eq__(self, other: "Event") -> bool:
        # Allow 150ms difference in timestamp
        return type(self) == type(other) and self.timestamp == pytest.approx(
            other.timestamp, abs=150
        )

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "Event":
        event_type = obj.get("type")
        if event_type == "state-change":
            return BrowserWindowSnapshotChange.from_json(obj)
        elif event_type == "window-focused":
            return WindowFocused.from_json(obj)
        else:
            raise ValueError(f"Unknown event type: {event_type}")


@dataclass
class BrowserWindowSnapshotChange(Event):
    is_incognito: bool
    url: str
    title: str

    def __init__(self, url: str, title: str, is_incognito: bool):
        super().__init__(unix_ms())
        self.url = url
        self.title = title
        self.is_incognito = is_incognito

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "BrowserWindowSnapshotChange":
        return BrowserWindowSnapshotChange(
            url=obj["url"], title=obj["title"], is_incognito=obj["isIncognito"]
        )


@dataclass
class WindowFocused(Event):
    window: int

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "WindowFocused":
        return WindowFocused(window=obj["window"], timestamp=obj["timestamp"])


EventType = Union[BrowserWindowSnapshotChange, WindowFocused]


class RecordedEvents:
    def __init__(self):
        self.events = []
        self.start = unix_ms()

    def push(self, event: Event):
        self.events.append(event)

    def __eq__(self, other):
        if not isinstance(other, RecordedEvents):
            return False
        if len(self.events) != len(other.events):
            return False
        for a, b in zip(self.events, other.events):
            if a != b:
                return False
        return True


@pytest.fixture
def events():
    yield RecordedEvents()


def unix_ms():
    return int(time.time_ns() / 1_000_000)
