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
    -- 1. Active / Inactive フィルター
    (($filter_flags & 0b011) = 0 
      OR (($filter_flags & 0b001) != 0 AND active = true)
      OR (($filter_flags & 0b010) != 0 AND active = false))
    
    -- 2. Priority フィルター
    AND (($filter_flags & 0b11100) = 0 
      OR ((1 << (priority + 2)) & $filter_flags) != 0)

    -- 3. Status フィルター
    AND (($filter_flags & 0b111100000) = 0 
      OR ((1 << (status + 5)) & $filter_flags) != 0)

-- 5. 動的ソートロジック (OrderFlags のビット判定)
ORDER BY 
    CASE 
        -- 昇順 (Reversed: 0b100000000 が立っていない場合)
        WHEN ($order_flags & 0b100000000) = 0 THEN
            CASE
                WHEN ($order_flags & 0b000000001) != 0 THEN status::VARCHAR
                WHEN ($order_flags & 0b000000010) != 0 THEN start_date
                WHEN ($order_flags & 0b000000100) != 0 THEN due_date
                WHEN ($order_flags & 0b000001000) != 0 THEN entry
                WHEN ($order_flags & 0b000010000) != 0 THEN end
                WHEN ($order_flags & 0b000100000) != 0 THEN priority::VARCHAR
                WHEN ($order_flags & 0b001000000) != 0 THEN progress::VARCHAR
                WHEN ($order_flags & 0b010000000) != 0 THEN time_spent::VARCHAR
                ELSE id::VARCHAR -- デフォルトソート
            END
    END ASC,
    CASE 
        -- 降順 (Reversed: 0b100000000 が立っている場合)
        WHEN ($order_flags & 0b100000000) != 0 THEN
            CASE
                WHEN ($order_flags & 0b000000001) != 0 THEN status::VARCHAR
                WHEN ($order_flags & 0b000000010) != 0 THEN start_date
                WHEN ($order_flags & 0b000000100) != 0 THEN due_date
                WHEN ($order_flags & 0b000001000) != 0 THEN entry
                WHEN ($order_flags & 0b000100000) != 0 THEN end
                WHEN ($order_flags & 0b001000000) != 0 THEN priority::VARCHAR
                WHEN ($order_flags & 0b010000000) != 0 THEN progress::VARCHAR
                WHEN ($order_flags & 0b100000000) != 0 THEN time_spent::VARCHAR
                ELSE id::VARCHAR
            END
    END DESC;
