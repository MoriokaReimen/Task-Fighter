SELECT 
  uuid, 
  active,
  project, 
  title, 
  detail, 
  priority, 
  start_day,
  due_day
FROM weekly_tasks 
WHERE 
  -- 1. Active Flags (1=True, 2=False)
  (
    (($filter_flags & 1) != 0 AND active = true)
    OR 
    (($filter_flags & 2) != 0 AND active = false)
  )
  AND 
  -- 2. Priority Flags (4=P0, 8=P1, 16=P2)
  (
    (($filter_flags & 4) != 0 AND priority = 0)
    OR
    (($filter_flags & 8) != 0 AND priority = 1)
    OR
    (($filter_flags & 16) != 0 AND priority = 2)
  )
-- 5. Sorting
ORDER BY 
  -- Ascending Sort
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 32) != 0  THEN priority END ASC,
  CASE WHEN ($order_flags & 256) = 0 THEN uuid END ASC,

  -- Descending Sort
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 32) != 0  THEN priority END DESC,
  CASE WHEN ($order_flags & 256) != 0 THEN uuid END DESC;
