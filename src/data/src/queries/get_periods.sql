WITH
    -- Parameters
    params(range_start, range_end,
           min_focus_score, max_distractive_score,
           min_focus_usage_dur, min_distractive_usage_dur,
           max_focus_gap, max_distractive_gap) AS (SELECT ?, ?, ?, ?, ?, ?, ?, ?),

    -- Distractive Periods: Initial
    -- get DPs from continuous usages of distractive apps
    --   (switches between session of the same app and between
    --   distractive apps are grouped together to form one usage)
    -- filter each DP to make sure its duration exceeds min_distractive_usage_dur
    initial_dp
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) over (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when the previous row's end != this row's start (i.e., not contiguous), else
                               -- is_group_start=0.
                               coalesce(LAG(end) OVER (ORDER BY u.start) <> u.start, 1) AS is_group_start
                        FROM usages u
                                 INNER JOIN sessions s on s.id = u.session_id
                                 INNER JOIN apps a ON a.id = s.app_id
                                 LEFT JOIN tags t ON t.id = a.tag_id
                                 CROSS JOIN params p
                        WHERE
                          -- filter usage if not in range
                          -- TODO: cutoff at edges??
                            u.start < p.range_end
                          AND u.end > p.range_start
                          -- filter usage if not from distractive app
                          AND p.max_distractive_score > coalesce(t.score, 0)))
            GROUP BY group_number
            -- Filter out DPs less than min_distractive_usage_dur
            HAVING MAX(end) - MIN(start) >= (SELECT min_distractive_usage_dur FROM params)),

    -- Distractive Periods: Combined if 'Close'
    --  if adjacent DPs have a gap less than max_distractive_gap then combine them
    dp
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) over (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when this DP's start - the previous DP's end exceeds max_distractive_gap, else
                               -- is_group_start=0. this is the 'if close' metric.
                               coalesce(u.start - LAG(end) OVER (ORDER BY u.start) > p.max_distractive_gap,
                                        1) AS is_group_start
                        FROM initial_dp u
                                 CROSS JOIN params p))
            GROUP BY group_number),

    -- Focus Periods: Initial
    -- filter each usage if it intersects with any DP
    -- get FPs from continuous usages of focus apps
    --   (switches between session of the same app and between
    --   focus apps are grouped together to form one usage)
    -- filter each FP to make sure its duration exceeds min_focus_usage_dur
    initial_fp
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) over (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so LAG(end) is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when the previous row's end != this row's start (i.e., not contiguous), else
                               -- is_group_start=0.
                               coalesce(LAG(end) OVER (ORDER BY u.start) <> u.start, 1) AS is_group_start
                        FROM usages u
                                 INNER JOIN sessions s on s.id = u.session_id
                                 INNER JOIN apps a ON a.id = s.app_id
                                 LEFT JOIN tags t ON t.id = a.tag_id
                                 CROSS JOIN params p
                        WHERE
                          -- filter usage if not in range
                          -- TODO: cutoff at edges??
                            u.start < p.range_end
                          AND u.end > p.range_start
                          -- filter usage if not from focus app
                          AND p.min_focus_score < coalesce(t.score, 0)
                          -- filter usage if it intersects with any DP
                          AND NOT EXISTS (SELECT 1
                                          FROM dp
                                          WHERE dp.start < u.end
                                            AND dp.end > u.start)))
            GROUP BY group_number
            -- Filter out FPs less than min_focus_usage_dur
            HAVING MAX(end) - MIN(start) >= (SELECT min_focus_usage_dur FROM params)),

    -- Focus Periods: Combined if 'Close'
    --  if adjacent FPs have a gap less than max_focus_gap and
    --  only if that gap doesn't intersect with any DP, then combine them
    fp
        AS (SELECT
                -- get range. alternatively, a 'first'/'last' getter of a group would work
                MIN(start) AS start,
                MAX(end)   AS end
            -- double window to do the grouping
            -- ref: https://stackoverflow.com/questions/63473256/how-do-i-group-datetimes-with-a-sqlite-windowing-functionk
            FROM (SELECT start, end, SUM(is_group_start) over (ORDER BY start) as group_number
                  FROM (SELECT u.start,
                               u.end,
                               -- window is all usages in [range_start, range_end], so prev_end is NULL
                               -- only for the first usage in range, where is_group_start=1. is_group_start=1
                               -- when this FP's start - the previous FP's end exceeds max_focus_gap, else
                               -- is_group_start=0. this is the 'if close' metric.
                               coalesce(u.start - u.prev_end > p.max_focus_gap, 1)
                                   ||
                                   -- or, start a new group if any DP intersects with the gap [u.prev_end, u.start]
                                   -- i.e., don't combine these FPs together if any DP is between them
                               EXISTS (SELECT 1
                                       FROM dp
                                       WHERE dp.start < u.start
                                         AND dp.end > u.prev_end)
                                   AS is_group_start
                        FROM
                            -- push the LAG window into a subquery
                            (SELECT u.start,
                                    u.end,
                                    LAG(end) OVER (ORDER BY u.start) AS prev_end
                             FROM initial_fp u) u
                                CROSS JOIN params p))
            GROUP BY group_number)


SELECT start, end, 0 AS is_focused
FROM dp

UNION ALL

SELECT start, end, 1 AS is_focused
FROM fp