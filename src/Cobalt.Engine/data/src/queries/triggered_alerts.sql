WITH

target_apps(alert_id, alert_version, app_id) AS (
        SELECT al.id, al.version, coalesce(al.app_id, at.app_id)
        FROM alerts al
        LEFT JOIN _app_tags at
            ON al.tag_id = at.tag_id
),

range_start(alert_id, alert_version, range_start) AS (
    SELECT al.id, al.version,
        CASE
            WHEN al.time_frame = 0 THEN ?
            WHEN al.time_frame = 1 THEN ?
            ELSE ?
        END range_start
    FROM alerts al
),

dur(alert_id, alert_version, range_start, dur) AS (
    SELECT al.id, al.version, t.range_start, (SELECT
        COALESCE(SUM(u.end - MAX(u.start, t.range_start)), 0)
        FROM target_apps ta
        INNER JOIN sessions s ON s.app_id = ta.app_id
        INNER JOIN usages u ON u.session_id = s.id
        WHERE ta.alert_id = al.id
            AND ta.alert_version = al.version
            AND u.end > t.range_start) dur
    FROM alerts al
    INNER JOIN range_start t
        ON t.alert_id = al.id
        AND t.alert_version = al.version
)

SELECT al.*, ae.timestamp, (CASE WHEN al.app_id IS NOT NULL THEN (
    SELECT a.name FROM apps a WHERE a.id = al.app_id
) ELSE (
    SELECT t.name FROM tags t WHERE t.id = al.tag_id
) END) name
    FROM alerts al
    INNER JOIN dur d
        ON al.id = d.alert_id
        AND al.version = d.alert_version
    LEFT JOIN alert_events ae
        ON al.id = ae.alert_id
        AND al.version = ae.alert_version
        AND ae.timestamp >= d.range_start
    WHERE d.dur >= al.usage_limit
    GROUP BY al.id
    HAVING al.version = max(al.version)