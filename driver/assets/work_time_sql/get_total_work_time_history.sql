-- 1. 指定された期間の日付一覧（カレンダー）を自動生成
WITH date_range AS (
    SELECT CAST(day AS DATE) AS target_date
    FROM generate_series(
        CAST($start_date AS DATE), -- 開始日 (今日からN-1日前)
        CAST($end_date AS DATE),   -- 終了日 (今日)
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
    date BETWEEN CAST($start_date AS DATE) AND CAST($end_date AS DATE)
    GROUP BY 
        date
)

-- 3. 日付一覧に作業時間を左結合（LEFT JOIN）し、空の日は 0.0 に埋める
SELECT 
    dr.target_date AS date,
    COALESCE(ds.total_time, 0.0) AS total_time
FROM 
    date_range dr
LEFT JOIN 
    daily_summary ds ON dr.target_date = ds.work_date
ORDER BY 
    date ASC;

-- ../assets/work_time_sql/get_total_work_time_history.sql
WITH date_range AS (
    SELECT CAST(day AS DATE) AS target_date
    FROM generate_series(
        CAST($start_date AS DATE),
        CAST($end_date AS DATE),
        INTERVAL 1 DAY
    ) AS g(day)
),
daily_summary AS (
    SELECT 
        date AS work_date,
        SUM(time_spent) AS total_time
    FROM 
        work_time
    WHERE 
        date BETWEEN CAST($start_date AS DATE) AND CAST($end_date AS DATE)
    GROUP BY 
        date
)
SELECT 
    dr.target_date AS date,
    COALESCE(ds.total_time, 0.0)
FROM 
    date_range dr
LEFT JOIN 
    daily_summary ds ON dr.target_date = ds.work_date
ORDER BY 
    date ASC;


