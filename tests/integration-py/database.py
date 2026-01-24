from pathlib import Path
import sqlite3
from typing import Optional
import time
from constants import TICKS_PER_SECOND, WINDOWS_EPOCH_OFFSET


def db_time() -> int:
    """Return current time as Windows ticks (100-nanosecond intervals since 1601-01-01)"""
    # Windows ticks are 100-nanosecond intervals since 1601-01-01
    # Convert Unix timestamp to Windows ticks
    unix_time = time.time()
    windows_ticks = unix_time * TICKS_PER_SECOND + WINDOWS_EPOCH_OFFSET
    return int(windows_ticks)


class Database:
    conn: sqlite3.Connection

    def __init__(self, path: Path):
        self.conn = sqlite3.connect(path)

    def create_app(
        self,
        name: str,
        description: str,
        company: str,
        color: str,
        identity_tag: int,
        identity_text0: str,
        identity_text1: str,
        tag_id: Optional[int],
        created_at: int,
        updated_at: int,
    ) -> int:
        """Create an app record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT OR REPLACE INTO apps (name, description, company, color, tag_id, 
                             identity_tag, identity_text0, identity_text1, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
            (
                name,
                description,
                company,
                color,
                tag_id,
                identity_tag,
                identity_text0,
                identity_text1,
                created_at,
                updated_at,
            ),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_tag(
        self,
        name: str,
        color: str,
        score: int,
        created_at: int,
        updated_at: int,
    ) -> int:
        """Create a tag record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO tags (name, color, score, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
        """,
            (name, color, score, created_at, updated_at),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_session(
        self,
        app_id: int,
        title: str,
        url: Optional[str],
    ) -> int:
        """Create a session record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO sessions (app_id, title, url)
            VALUES (?, ?, ?)
        """,
            (app_id, title, url),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_usage(
        self,
        session_id: int,
        start: int,
        end: int,
    ) -> int:
        """Create a usage record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO usages (session_id, start, end)
            VALUES (?, ?, ?)
        """,
            (session_id, start, end),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_interaction_period(
        self,
        start: int,
        end: int,
        mouse_clicks: int,
        key_strokes: int,
    ) -> int:
        """Create an interaction period record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO interaction_periods (start, end, mouse_clicks, key_strokes)
            VALUES (?, ?, ?, ?)
        """,
            (start, end, mouse_clicks, key_strokes),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_system_event(
        self,
        timestamp: int,
        event: int,
    ) -> int:
        """Create a system event record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO system_events (timestamp, event)
            VALUES (?, ?)
        """,
            (timestamp, event),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_alert(
        self,
        app_id: Optional[int],
        tag_id: Optional[int],
        usage_limit: int,
        time_frame: int,
        trigger_action_tag: int,
        trigger_action_dim_duration: Optional[int],
        trigger_action_message_content: Optional[str],
        active: bool,
        created_at: int,
        updated_at: int,
    ) -> int:
        """Create an alert record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO alerts (app_id, tag_id, usage_limit, time_frame, 
                               trigger_action_tag, trigger_action_dim_duration, 
                               trigger_action_message_content, active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
            (
                app_id,
                tag_id,
                usage_limit,
                time_frame,
                trigger_action_tag,
                trigger_action_dim_duration,
                trigger_action_message_content,
                active,
                created_at,
                updated_at,
            ),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_reminder(
        self,
        alert_id: int,
        threshold: float,
        message: str,
        active: bool,
        created_at: int,
        updated_at: int,
    ) -> int:
        """Create a reminder record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO reminders (alert_id, threshold, message, active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
        """,
            (alert_id, threshold, message, active, created_at, updated_at),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_alert_event(
        self,
        alert_id: int,
        timestamp: int,
        reason: int,
    ) -> int:
        """Create an alert event record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO alert_events (alert_id, timestamp, reason)
            VALUES (?, ?, ?)
        """,
            (alert_id, timestamp, reason),
        )
        self.conn.commit()
        return cursor.lastrowid

    def create_reminder_event(
        self,
        reminder_id: int,
        timestamp: int,
        reason: int,
    ) -> int:
        """Create a reminder event record and return its ID"""
        cursor = self.conn.cursor()
        cursor.execute(
            """
            INSERT INTO reminder_events (reminder_id, timestamp, reason)
            VALUES (?, ?, ?)
        """,
            (reminder_id, timestamp, reason),
        )
        self.conn.commit()
        return cursor.lastrowid
