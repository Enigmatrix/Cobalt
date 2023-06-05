EXPLAIN QUERY PLAN WITH

target_app(id, alert, name, identity_tag, identity_text0) AS (
    SELECT a.id, al.id, a.name, a.identity_tag, a.identity_text0 FROM alert al
    INNER JOIN app a ON CASE
        WHEN al.target_is_app THEN a.id = al.app
        ELSE a.id IN (
            SELECT at.app
            FROM _app_tag at
            WHERE al.tag = at.tag)
        END
),

start(id, start) AS (
    SELECT id,
        CASE
            WHEN a.time_frame = 0 THEN ?
            WHEN a.time_frame = 1 THEN ?
            ELSE ?
        END
    FROM alert a
),

dur(id, dur) AS (
    SELECT al.id, (SELECT COALESCE(SUM(u.end - MAX(u.start, t.start)), 0) dur
        FROM target_app ta
        INNER JOIN session s ON s.app = ta.id
        INNER JOIN usage u ON u.session = s.id
        WHERE ta.alert = al.id AND u.end > t.start)
    FROM alert al
    INNER JOIN start AS t ON t.id = al.id
),

last_reminder_hit(id, timestamp) AS (
    SELECT rh.reminder, MAX(timestamp)
    FROM reminder_hit rh
    GROUP BY rh.reminder
)

SELECT
    0,
    ta.alert,
    ta.id,
    ta.name,
    ta.identity_tag,
    ta.identity_text0,
    al.action_tag,
    al.action_int0,
    al.action_text0
FROM alert al
INNER JOIN dur d ON d.id = al.id
INNER JOIN target_app ta ON ta.alert = al.id
WHERE d.dur >= al.usage_limit

UNION ALL

SELECT
    1,
    al.id,
    r.id,
    d.dur,
    al.usage_limit,
    r.threshold,
    r.message,
    NULL,
    NULL
FROM reminder r
INNER JOIN alert al ON al.id = r.alert
INNER JOIN dur d ON d.id = al.id
INNER JOIN start s ON s.id = al.id
LEFT JOIN last_reminder_hit lrh ON lrh.id = r.id
WHERE dur >= al.usage_limit * r.threshold
    AND COALESCE(lrh.timestamp, 0) < s.start
