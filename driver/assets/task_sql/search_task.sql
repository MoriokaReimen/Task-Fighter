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
  (($filter_flags & '0b011'::INTEGER) = '0b000'::INTEGER
    OR (($filter_flags & '0b001'::INTEGER) != '0b000'::INTEGER AND active = true)
    OR (($filter_flags & '0b010'::INTEGER) != '0b000'::INTEGER AND active = false))
  
  -- 2. Priority フィルター (シフト演算結果をBITにしてからAND)
  AND (($filter_flags & '0b11100'::INTEGER) = '0b00000'::INTEGER 
    OR (((1 << (priority + 2)) & $filter_flags) != '0b00000'::INTEGER))

  -- 3. Status フィルター (シフト演算結果をBITにしてからAND)
  AND (($filter_flags & '0b111100000'::INTEGER) = '0b000000000'::INTEGER 
    OR (((1 << (status + 5)) & $filter_flags) != '0b000000000'::INTEGER))

  -- 4. 検索テキストフィルター (Regexp / LIKE)
  AND ($pattern = '' OR (
    (($search_flags & '0b0001'::INTEGER) != '0b0000'::INTEGER AND (CASE WHEN ($search_flags & '0b1000'::INTEGER) != '0b0000'::INTEGER THEN regexp_matches(title, $pattern) ELSE title LIKE '%' || $pattern || '%' END))
    OR (($search_flags & '0b0010'::INTEGER) != '0b0000'::INTEGER AND (CASE WHEN ($search_flags & '0b1000'::INTEGER) != '0b0000'::INTEGER THEN regexp_matches(project, $pattern) ELSE project LIKE '%' || $pattern || '%' END))
    OR (($search_flags & '0b0100'::INTEGER) != '0b0000'::INTEGER AND (CASE WHEN ($search_flags & '0b1000'::INTEGER) != '0b0000'::INTEGER THEN regexp_matches(detail, $pattern) ELSE detail LIKE '%' || $pattern || '%' END))
  ))

-- 5. 動的ソートロジック
ORDER BY 
    CASE 
        -- 昇順 (Reversedフラグ が立っていない場合)
        WHEN ($order_flags & '0b100000000'::INTEGER) = '0b000000000'::INTEGER THEN
            CASE
                WHEN ($order_flags & '0b000000001'::INTEGER) != '0b000000000'::INTEGER THEN status::VARCHAR
                WHEN ($order_flags & '0b000000010'::INTEGER) != '0b000000000'::INTEGER THEN start_date::VARCHAR
                WHEN ($order_flags & '0b000000100'::INTEGER) != '0b000000000'::INTEGER THEN due_date::VARCHAR
                WHEN ($order_flags & '0b000001000'::INTEGER) != '0b000000000'::INTEGER THEN entry_date::VARCHAR
                WHEN ($order_flags & '0b000010000'::INTEGER) != '0b000000000'::INTEGER THEN end_date::VARCHAR
                WHEN ($order_flags & '0b000100000'::INTEGER) != '0b000000000'::INTEGER THEN priority::VARCHAR
                WHEN ($order_flags & '0b001000000'::INTEGER) != '0b000000000'::INTEGER THEN progress::VARCHAR
                WHEN ($order_flags & '0b010000000'::INTEGER) != '0b000000000'::INTEGER THEN time_spent::VARCHAR
                ELSE id::VARCHAR
            END
    END ASC,
    CASE 
        -- 降順 (Reversedフラグ が立っている場合)
        WHEN ($order_flags & '0b100000000'::INTEGER) != '0b000000000'::INTEGER THEN
            CASE
                WHEN ($order_flags & '0b000000001'::INTEGER) != '0b000000000'::INTEGER THEN status::VARCHAR
                WHEN ($order_flags & '0b000000010'::INTEGER) != '0b000000000'::INTEGER THEN start_date::VARCHAR
                WHEN ($order_flags & '0b000000100'::INTEGER) != '0b000000000'::INTEGER THEN due_date::VARCHAR
                WHEN ($order_flags & '0b000001000'::INTEGER) != '0b000000000'::INTEGER THEN entry_date::VARCHAR
                WHEN ($order_flags & '0b000010000'::INTEGER) != '0b000000000'::INTEGER THEN end_date::VARCHAR
                WHEN ($order_flags & '0b000100000'::INTEGER) != '0b000000000'::INTEGER THEN priority::VARCHAR
                WHEN ($order_flags & '0b001000000'::INTEGER) != '0b000000000'::INTEGER THEN progress::VARCHAR
                WHEN ($order_flags & '0b010000000'::INTEGER) != '0b000000000'::INTEGER THEN time_spent::VARCHAR
                ELSE id::VARCHAR
            END
    END DESC;
