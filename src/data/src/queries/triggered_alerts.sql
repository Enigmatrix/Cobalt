WITH

target_apps(alert_id, app_id) AS (
        SELECT al.id, coalesce(al.app_id, a.id)
        FROM alerts al
        LEFT JOIN apps a
            ON al.tag_id = a.tag_id
            AND a.tag_id IS NOT NULL
),

range_start(alert_id, range_start) AS (
    SELECT al.id,
        CASE
            WHEN al.time_frame = 0 THEN ?
            WHEN al.time_frame = 1 THEN ?
            ELSE ?
        END range_start
    FROM alerts al
),

dur(alert_id, range_start, dur) AS (
    SELECT al.id, t.range_start, (SELECT
        COALESCE(SUM(u.end - MAX(u.start, t.range_start)), 0)
        FROM target_apps ta
        INNER JOIN sessions s ON s.app_id = ta.app_id
        INNER JOIN usages u ON u.session_id = s.id
        WHERE ta.alert_id = al.id
            AND u.end > t.range_start) dur
    FROM alerts al
    INNER JOIN range_start t
        ON t.alert_id = al.id
)

SELECT al.*, ae.timestamp, (CASE WHEN al.app_id IS NOT NULL THEN (
    SELECT a.name FROM apps a WHERE a.id = al.app_id
) ELSE (
    SELECT t.name FROM tags t WHERE t.id = al.tag_id
) END) name
    FROM alerts al
    INNER JOIN dur d
        ON al.id = d.alert_id
    LEFT JOIN alert_events ae
        ON al.id = ae.alert_id
        AND ae.timestamp >= d.range_start
    WHERE d.dur >= al.usage_limit
    GROUP BY al.id
    HAVING al.prev IS NULL