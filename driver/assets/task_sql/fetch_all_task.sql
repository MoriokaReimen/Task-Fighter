SELECT 
  uuid,
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
    -- Active Flags
    (
        (($filter_flags & '0b000000001'::INTEGER) != 0 AND active = true)
        OR 
        (($filter_flags & '0b000000010'::INTEGER) != 0 AND active = false)
    )
    AND 
    -- Priority Flags
    (
        (($filter_flags & '0b000000100'::INTEGER) != 0 AND priority = 0)
        OR
        (($filter_flags & '0b000001000'::INTEGER) != 0 AND priority = 1)
        OR
        (($filter_flags & '0b000010000'::INTEGER) != 0 AND priority = 2)
    )
    AND
    (
    -- Status Flags
        (($filter_flags & '0b000100000'::INTEGER) != 0 AND status = 0)
        OR
        (($filter_flags & '0b001000000'::INTEGER) != 0 AND status = 1)
        OR
        (($filter_flags & '0b010000000'::INTEGER) != 0 AND status = 2)
        OR
        (($filter_flags & '0b100000000'::INTEGER) != 0 AND status = 3)
    )
-- 5. 動的ソートロジック
ORDER BY 
-- 昇順かつ各フラグがマッチしたときだけそのカラムでソート（それ以外はNULL＝ソートに影響しない）
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 1) != 0   THEN status END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 2) != 0   THEN start_date END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 4) != 0   THEN due_date END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 8) != 0   THEN entry_date END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 16) != 0  THEN end_date END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 32) != 0  THEN priority END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 64) != 0  THEN progress END ASC,
CASE WHEN ($order_flags & 256) = 0 AND ($order_flags & 128) != 0 THEN time_spent END ASC,
-- どのフラグも立っていない場合のデフォルト
CASE WHEN ($order_flags & 256) = 0 THEN uuid END ASC,
-- 昇順かつ各フラグがマッチしたときだけそのカラムでソート（それ以外はNULL＝ソートに影響しない）
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 1) != 0   THEN status END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 2) != 0   THEN start_date END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 4) != 0   THEN due_date END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 8) != 0   THEN entry_date END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 16) != 0  THEN end_date END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 32) != 0  THEN priority END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 64) != 0  THEN progress END DESC,
CASE WHEN ($order_flags & 256) != 0 AND ($order_flags & 128) != 0 THEN time_spent END DESC,
-- どのフラグも立っていない場合のデフォルト
CASE WHEN ($order_flags & 256) != 0 THEN uuid END DESC;

