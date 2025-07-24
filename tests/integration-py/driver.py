import time
import sys
import pytest
import logging
import tempfile
import shutil
import subprocess
import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import Union, Optional, Dict, Any, List
import difflib
from wasabi import color

logger = logging.getLogger(__name__)

TOLERANCE_MS = 500


class DriverData:
    def __init__(self, root_dir: Path, save_dir: Path):
        self.root_dir = root_dir
        self.save_dir = save_dir
        self.file_path = save_dir / "data.json"

        self.driver_path = root_dir / "target" / "debug" / "driver_web_state.exe"
        self.stdout_path = save_dir / "driver_web_state_stdout.txt"
        self.stderr_path = save_dir / "driver_web_state_stderr.txt"
        self.driver_program = None

    def start(self):
        logger.info(f"Starting driver_web_state from {self.driver_path}...")
        # Start driver_web_state in background, save stdout/stderr to files
        self.driver_program = subprocess.Popen(
            [str(self.driver_path)],
            cwd=self.save_dir,
            stdout=open(self.stdout_path, "w"),
            stderr=open(self.stderr_path, "w"),
            text=True,
            bufsize=0,
        )

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

    data = DriverData(root, save_dir)
    yield data

    # Cleanup
    try:
        if data.driver_program:
            data.driver_program.kill()
    except Exception as e:
        logger.warning(f"Failed to terminate driver_program: {e}")

    # Send output to stdout/stderr
    sys.stdout.write("--- driver_web_state stdout ---\n")
    try:
        out = open(data.stdout_path, "rb").read()
        sys.stdout.buffer.write(out)
    except Exception as e:
        logger.warning("Failed to read stdout of driver: %s", e, exc_info=True)
    sys.stderr.write("--- driver_web_state stderr ---")
    try:
        err = open(data.stderr_path, "rb").read()
        sys.stderr.buffer.write(err)
    except Exception as e:
        logger.warning("Failed to read stderr of driver: %s", e, exc_info=True)

    shutil.rmtree(save_dir, ignore_errors=True)


# Unix epoch timestamp in milliseconds, with {TOLERANCE_MS}ms tolerance
class Timestamp:
    def __init__(self, timestamp: Optional[int] = None):
        self.timestamp = timestamp or unix_ms()

    def __eq__(self, other: "Timestamp") -> bool:
        return self.timestamp == pytest.approx(other.timestamp, abs=TOLERANCE_MS)

    def __cmp__(self, other: "Timestamp") -> int:
        raise NotImplementedError("Timestamp comparison is not supported")

    def __str__(self) -> str:
        return f"{self.timestamp}"

    def __repr__(self) -> str:
        return f"{self.timestamp}"


@dataclass
class Event:
    timestamp: Timestamp = field(default_factory=lambda: Timestamp(), kw_only=True)

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "Event":
        event_type = obj.get("type")
        if event_type == "state-change":
            return Change.from_json(obj)
        elif event_type == "window-focused":
            return WindowFocused.from_json(obj)
        else:
            raise ValueError(f"Unknown event type: {event_type}")


@dataclass
class Change(Event):
    url: str
    title: str
    is_incognito: bool = False

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "Change":
        return Change(
            timestamp=Timestamp(obj["timestamp"]),
            url=obj["url"],
            title=obj["title"],
            is_incognito=obj["isIncognito"],
        )


@dataclass
class WindowFocused(Event):
    window: int

    @staticmethod
    def from_json(obj: Dict[str, Any]) -> "WindowFocused":
        return WindowFocused(
            timestamp=Timestamp(obj["timestamp"]), window=obj["window"]
        )


EventType = Union[Change, WindowFocused]


class RecordedEvents:
    def __init__(self):
        self.events = []
        self.start = unix_ms()

    def push(self, event: Event):
        self.events.append(event)

    def __eq__(self, other: List[Event]):
        if len(self.events) != len(other):
            logger.error(f"Event length mismatch: {len(self.events)} != {len(other)}")
            self.diff_events(other)
            return False

        for a, b in zip(self.events, other):
            if a != b:
                logger.error(f"Event mismatch: {a} != {b}")
                self.diff_events(other)
                assert a == b  # will fail
                return False
        return True

    def diff_events(self, other: List[Event]):
        list1 = "\n".join(str(e) for e in self.events)
        list2 = "\n".join(str(e) for e in other)

        def diff_strings(a, b):
            output = []
            matcher = difflib.SequenceMatcher(None, a, b)
            for opcode, a0, a1, b0, b1 in matcher.get_opcodes():
                if opcode == "equal":
                    output.append(a[a0:a1])
                elif opcode == "insert":
                    output.append(color(b[b0:b1], fg=16, bg="green"))
                elif opcode == "delete":
                    output.append(color(a[a0:a1], fg=16, bg="red"))
                elif opcode == "replace":
                    output.append(color(a[a0:a1], fg=16, bg="red"))
                    output.append(color(b[b0:b1], fg=16, bg="green"))
            return "".join(output)

        d = diff_strings(list1, list2)
        logger.info("--- Event diff (actual vs expected) ---\n" + d)


@pytest.fixture
def events():
    yield RecordedEvents()


def unix_ms():
    return int(time.time_ns() / 1_000_000)
