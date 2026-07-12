SELECT 
  id, 
  active,
  project, 
  title, 
  detail, 
  priority
FROM daily_tasks 
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
  -- 4. Search Flags
  AND ($pattern = '' OR (
    -- Title Search (1=Match, 8=RegEx)
    (($search_flags & 1) != 0 AND ((($search_flags & 8) != 0 AND regexp_matches(title, $pattern)) OR (($search_flags & 8) = 0 AND title LIKE '%' || $pattern || '%')))
    -- Project Search (2=Match, 8=RegEx)
    OR (($search_flags & 2) != 0 AND ((($search_flags & 8) != 0 AND regexp_matches(project, $pattern)) OR (($search_flags & 8) = 0 AND project LIKE '%' || $pattern || '%')))
    -- Detail Search (4=Match, 8=RegEx)
    OR (($search_flags & 4) != 0 AND ((($search_flags & 8) != 0 AND regexp_matches(detail, $pattern)) OR (($search_flags & 8) = 0 AND detail LIKE '%' || $pattern || '%')))
  ))
-- 5. 動的ソートロジック
ORDER BY
  -- Ascending Sort
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 32) != 0  THEN priority END ASC,
  CASE WHEN ($order_flags & 256) = 0 THEN id END ASC,

  -- Descending Sort
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 32) != 0  THEN priority END DESC,
  CASE WHEN ($order_flags & 256) != 0 THEN id END DESC;
