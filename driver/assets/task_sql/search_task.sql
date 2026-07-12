SELECT 
  id, 
  active, 
  status, 
  project, 
  title, 
  detail, 
  start_date::VARCHAR AS start_date,
  due_date::VARCHAR AS due_date,
  priority, 
  progress, 
  time_spent, 
  entry_date::VARCHAR AS entry_date,
  end_date::VARCHAR AS end_date
FROM tasks 
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
  AND
  -- 3. Status Flags (32=S0, 64=S1, 128=S2, 256=S3)
  (
    (($filter_flags & 32) != 0 AND status = 0)
    OR
    (($filter_flags & 64) != 0 AND status = 1)
    OR
    (($filter_flags & 128) != 0 AND status = 2)
    OR
    (($filter_flags & 256) != 0 AND status = 3)
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

ORDER BY 
  -- 【昇順ソート】型ごとにCASEを分けることで正しい順序でソート
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 1) != 0   THEN status END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 2) != 0   THEN start_date END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 4) != 0   THEN due_date END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 8) != 0   THEN entry_date END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 16) != 0  THEN end_date END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 32) != 0  THEN priority END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 64) != 0  THEN progress END ASC,
  CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 128) != 0 THEN time_spent END ASC,
  CASE WHEN ($order_flags & 256) = 0 THEN id END ASC,

  -- 【降順ソート】
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 1) != 0   THEN status END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 2) != 0   THEN start_date END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 4) != 0   THEN due_date END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 8) != 0   THEN entry_date END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 16) != 0  THEN end_date END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 32) != 0  THEN priority END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 64) != 0  THEN progress END DESC,
  CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 128) != 0 THEN time_spent END DESC,
  CASE WHEN ($order_flags & 256) != 0 THEN id END DESC;
