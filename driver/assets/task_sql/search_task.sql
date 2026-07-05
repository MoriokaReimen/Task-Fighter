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
  -- 1. Active / Inactive フィルター (Active=1, Inactive=2)
  (($filter_flags & 0b011) = 0 
  OR (($filter_flags & 0b001) != 0 AND active = true)
  OR (($filter_flags & 0b010) != 0 AND active = false))

  -- 2. Priority フィルター (Low=4, Middle=8, High=16)
  AND (($filter_flags & 0b11100) = 0 
  OR ((1 << (priority + 2)) & $filter_flags) != 0)

  -- 3. Status フィルター (Pending=32, WIP=64, Complete=128, Canceled=256)
  AND (($filter_flags & 0b111100000) = 0 
  OR ((1 << (status + 5)) & $filter_flags) != 0)

  -- 4. 動的検索ロジック (SearchTitle=1, SearchProject=2, SearchDetail=4, EnableRegex=8)
  AND ($pattern = '' OR (
  (($search_flags & 0b0001) != 0 AND (CASE WHEN ($search_flags & 0b1000) != 0 THEN regexp_matches(title, $pattern) ELSE title LIKE '%' || $pattern || '%' END))
  OR (($search_flags & 0b0010) != 0 AND (CASE WHEN ($search_flags & 0b1000) != 0 THEN regexp_matches(project, $pattern) ELSE project LIKE '%' || $pattern || '%' END))
  OR (($search_flags & 0b0100) != 0 AND (CASE WHEN ($search_flags & 0b1000) != 0 THEN regexp_matches(detail, $pattern) ELSE detail LIKE '%' || $pattern || '%' END))
  ))

-- 5. 動的ソートロジック (OrderFlags のビット判定)
ORDER BY 
-- 5a. 昇順ソート用ブロック (Reversed: 0b100000000 が立っていない場合のみ評価)
CASE WHEN ($order_flags & 0b100000000) = 0 THEN
CASE
WHEN ($order_flags & 0b000000001) != 0 THEN status::VARCHAR
WHEN ($order_flags & 0b000000010) != 0 THEN start_date
WHEN ($order_flags & 0b000000100) != 0 THEN due_date
WHEN ($order_flags & 0b000001000) != 0 THEN entry
WHEN ($order_flags & 0b000010000) != 0 THEN end
WHEN ($order_flags & 0b001000000) != 0 THEN priority::VARCHAR
WHEN ($order_flags & 0b010000000) != 0 THEN progress::VARCHAR
WHEN ($order_flags & 0b100000000) != 0 THEN time_spent::VARCHAR
ELSE id::VARCHAR
END
END ASC,

-- 5b. 降順ソート用ブロック (Reversed: 0b100000000 が立っている場合のみ評価)
CASE WHEN ($order_flags & 0b100000000) != 0 THEN
CASE
WHEN ($order_flags & 0b000000001) != 0 THEN status::VARCHAR
WHEN ($order_flags & 0b000000010) != 0 THEN start_date
WHEN ($order_flags & 0b000000100) != 0 THEN due_date
WHEN ($order_flags & 0b000001000) != 0 THEN entry
WHEN ($order_flags & 0b000010000) != 0 THEN end
WHEN ($order_flags & 0b001000000) != 0 THEN priority::VARCHAR
WHEN ($order_flags & 0b010000000) != 0 THEN progress::VARCHAR
WHEN ($order_flags & 0b100000000) != 0 THEN time_spent::VARCHAR
ELSE id::VARCHAR
END
END DESC;
