SELECT 
  id, 
  active,
  project, 
  title, 
  detail, 
  priority, 
FROM daily_tasks 
WHERE 
    -- 1. Active / Inactive フィルター (両辺をBIT型にして演算)
    (($filter_flags & '0b011'::INTEGER) = '0b000'::INTEGER
      OR (($filter_flags & '0b001'::INTEGER) != '0b000'::INTEGER AND active = true)
      OR (($filter_flags & '0b010'::INTEGER) != '0b000'::INTEGER AND active = false))
    -- 2. Priority フィルター (1 << (priority + 2) の結果をBIT型にキャスト)
    AND (($filter_flags & '0b11100'::INTEGER) = '0b00000'::INTEGER 
      OR (((1 << (priority + 2)) & $filter_flags) != '0b00000'::INTEGER))

-- 5. 動的ソートロジック
ORDER BY 
    CASE 
        -- 昇順 (Reversedフラグが立っていない場合)
        WHEN ($order_flags & '0b100000000'::INTEGER) = '0b000000000'::INTEGER THEN
            CASE
                WHEN ($order_flags & '0b000100000'::INTEGER) != '0b000000000'::INTEGER THEN priority::VARCHAR
                ELSE id::VARCHAR
            END
    END ASC,
    CASE 
        -- 降順 (Reversedフラグが立っている場合)
        WHEN ($order_flags & '0b100000000'::INTEGER) != '0b000000000'::INTEGER THEN
            CASE
                WHEN ($order_flags & '0b000100000'::INTEGER) != '0b000000000'::INTEGER THEN priority::VARCHAR
                ELSE id::VARCHAR
            END
    END DESC;
