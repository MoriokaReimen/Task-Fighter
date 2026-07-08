WITH actual_start AS (
SELECT GREATEST(
  CAST($start_date AS DATE), 
  COALESCE(MIN(start_date), CAST($start_date AS DATE))
) AS start_d 
FROM tasks
),
date_range AS (
SELECT CAST(d AS DATE) AS d
FROM generate_series(
  (SELECT start_d FROM actual_start), 
  CAST($end_date AS DATE), 
  INTERVAL 1 DAY
) AS t(d)
)
SELECT 
SUM(CASE WHEN t.status = 0 THEN 1 ELSE 0 END) AS pending,
SUM(CASE WHEN t.status = 1 THEN 1 ELSE 0 END) AS wip,
SUM(CASE WHEN t.status = 2 THEN 1 ELSE 0 END) AS complete,
SUM(CASE WHEN t.status = 3 THEN 1 ELSE 0 END) AS canceled
FROM date_range r
LEFT JOIN tasks t ON t.start_date <= r.d AND (t.end_date >= r.d OR t.end_date IS NULL)
GROUP BY r.d
ORDER BY r.d DESC
LIMIT 100;
