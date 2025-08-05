WITH
    -- Parameters
    params(range_start, range_end,
           min_focus_score, max_distractive_score,
           min_focus_usage_dur, min_distractive_usage_dur,
           max_focus_gap, max_distractive_gap) AS (SELECT ?, ?, ?, ?, ?, ?, ?, ?),

    -- Distractive Streaks (DSs): Initial
    -- get DSs from continuous usages of distractive apps
    --   (switches between session of the same app and between
    --   distractive apps are grouped together to form one usage)
    -- filter each DS to make sure its duration exceeds min_distractive_usage_dur
    initial_ds
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) OVER (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when the previous row's end != this row's start (i.e., not contiguous), else
                               -- is_group_start=0.
                               coalesce(LAG(end) OVER (ORDER BY u.start) <> u.start, 1) AS is_group_start
                        FROM (SELECT u.start, u.end, u.session_id FROM usages u) u
                                 INNER JOIN sessions s on s.id = u.session_id
                                 INNER JOIN apps a ON a.id = s.app_id
                                 LEFT JOIN tags t ON t.id = a.tag_id
                                 CROSS JOIN params p
                        WHERE
                          -- filter usage if not in range
                            u.start < p.range_end
                          AND u.end > p.range_start
                          -- filter usage if not from distractive app
                          AND p.max_distractive_score > coalesce(t.score, 0)))
            GROUP BY group_number
            -- Filter out DSs less than min_distractive_usage_dur
            HAVING MAX(end) - MIN(start) >= (SELECT min_distractive_usage_dur FROM params)),

    -- Distractive Streaks: Combined if 'Close'
    --  if adjacent DSs have a gap less than max_distractive_gap then combine them
    ds
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) OVER (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when this DS's start - the previous DS's end exceeds max_distractive_gap, else
                               -- is_group_start=0. this is the 'if close' metric.
                               coalesce(u.start - LAG(end) OVER (ORDER BY u.start) > p.max_distractive_gap,
                                        1) AS is_group_start
                        FROM initial_ds u
                                 CROSS JOIN params p))
            GROUP BY group_number),

    -- Focus Streaks: Initial
    -- filter each usage if it intersects with any DS
    -- get FSs from continuous usages of focus apps
    --   (switches between session of the same app and between
    --   focus apps are grouped together to form one usage)
    -- filter each FS to make sure its duration exceeds min_focus_usage_dur
    initial_fs
        AS (SELECT u.start, u.end
            FROM (SELECT
                      -- get range. alternatively, a 'first'/'last' getter of a group would work
                      MIN(start) AS start,
                      MAX(end)   AS end
                  -- double window to do the grouping
                  -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
                  FROM (SELECT start, end, SUM(is_group_start) OVER (ORDER BY start) as group_number
                        FROM (SELECT u.start,
                                     u.end,
                                     -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                                     -- only for the first usage in range, where is_group_start=1. is_group_start=1
                                     -- when the previous row's end != this row's start (i.e., not contiguous), else
                                     -- is_group_start=0.
                                     coalesce(LAG(end) OVER (ORDER BY u.start) <> u.start, 1) AS is_group_start
                              FROM (SELECT u.start, u.end, u.session_id FROM usages u) u
                                       INNER JOIN sessions s on s.id = u.session_id
                                       INNER JOIN apps a ON a.id = s.app_id
                                       LEFT JOIN tags t ON t.id = a.tag_id
                                       CROSS JOIN params p
                              WHERE
                                -- filter usage if not in range
                                  u.start < p.range_end
                                AND u.end > p.range_start
                                -- filter usage if not from focus app
                                AND p.min_focus_score < coalesce(t.score, 0)))
                  GROUP BY group_number) u
            -- Filter out FSs less than min_focus_usage_dur
            WHERE u.end - u.start >= (SELECT min_focus_usage_dur FROM params)
              -- filter out FSs that intersects with any DS
              AND NOT EXISTS (SELECT 1
                              FROM ds
                              WHERE ds.start < u.end
                                AND ds.end > u.start)),

    -- Focus Streaks: Combined if 'Close'
    --  if adjacent FSs have a gap less than max_focus_gap and
    --  only if that gap doesn't intersect with any DS, then combine them
    fs
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) OVER (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so prev_end is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when this FS's start - the previous FS's end exceeds max_focus_gap, else
                               -- is_group_start=0. this is the 'if close' metric.
                               coalesce(u.start - u.prev_end > p.max_focus_gap, 1)
                                   ||
                                   -- or, start a new group if any DS intersects with the gap [u.prev_end, u.start]
                                   -- i.e., don't combine these FSs together if any DS is between them
                               EXISTS (SELECT 1
                                       FROM ds
                                       WHERE ds.start < u.start
                                         AND ds.end > u.prev_end)
                                   AS is_group_start
                        FROM
                            -- push the LAG window into a subquery
                            (SELECT u.start,
                                    u.end,
                                    LAG(end) OVER (ORDER BY u.start) AS prev_end
                             FROM initial_fs u) u
                                CROSS JOIN params p))
            GROUP BY group_number)


SELECT MAX(start, p.range_start) AS start,
       MIN(end, p.range_end)     AS end,
       0                         AS is_focused
FROM ds
         CROSS JOIN params p

UNION ALL

SELECT MAX(start, p.range_start) AS start,
       MIN(end, p.range_end)     AS end,
       1                         AS is_focused
FROM fs
         CROSS JOIN params p