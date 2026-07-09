-- 1. 指定された期間の日付一覧（カレンダー）を自動生成
WITH date_range AS (
    SELECT CAST(day AS DATE) AS target_date
    FROM generate_series(
        CAST($start_date AS DATE), -- $ から $ に変更
        CAST($end_date AS DATE),   -- $ から $ に変更
        INTERVAL 1 DAY
    ) AS g(day)
),

-- 2. 期間内の実際の作業時間を日毎に集計
daily_summary AS (
    SELECT 
        date AS work_date,
        SUM(time_spent) AS total_time
    FROM 
        work_time
    WHERE 
        date BETWEEN CAST($start_date AS DATE) AND CAST($end_date AS DATE) -- $ から $ に変更
    GROUP BY 
        date
)

-- 3. 日付一覧に作業時間を左結合（LEFT JOIN）し、空の日は 0.0 に埋める
SELECT 
    CAST(dr.target_date AS VARCHAR) AS date,
    COALESCE(ds.total_time, 0.0) AS total_time
FROM 
    date_range dr
LEFT JOIN 
    daily_summary ds ON dr.target_date = ds.work_date
ORDER BY 
    date ASC;
