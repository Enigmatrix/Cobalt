WITH

target_app(id, alert, name, identity_tag, identity_text0) AS (
    SELECT a.id, al.id, a.name, a.identity_tag, a.identity_text0 FROM app a
    INNER JOIN alert al ON CASE
        WHEN al.target_is_app THEN al.app = a.id
        ELSE (
            SELECT 1
            FROM _app_tag ta
            WHERE ta.app = a.id
                AND ta.tag = al.tag)
    END
),

start(id, start) AS (
    SELECT id,
        CASE
            WHEN a.time_frame = 0 THEN 0
            WHEN a.time_frame = 1 THEN 0
            ELSE 0
        END
    FROM alert a
),

dur(id, dur) AS (
    SELECT al.id, (
        SELECT COALESCE(SUM(u.end - MAX(u.start, t.start)), 0)
        FROM session s
        INNER JOIN usage u ON u.session = s.id
        WHERE s.app = ta.id AND u.end > t.start)
    FROM alert al
    INNER JOIN start AS t ON al.id = t.id
    INNER JOIN target_app ta ON ta.alert = al.id
),

last_reminder_hit(id, timestamp) AS (
    SELECT rh.id, COALESCE(MAX(rh.timestamp), 0)
    FROM reminder_hit rh
)

SELECT
    1,
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

UNION

SELECT
    0,
    al.id,
    r.id,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    r.message
FROM reminder r
INNER JOIN alert al ON al.id = r.alert
INNER JOIN dur d ON d.id = al.id
INNER JOIN last_reminder_hit lrh ON lrh.id = r.id
INNER JOIN start s ON s.id = al.id
WHERE dur >= al.usage_limit * r.threshold
    AND lrh.timestamp < s.start;
