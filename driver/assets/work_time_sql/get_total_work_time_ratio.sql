SELECT 
    task_id,
    SUM(time_spent) AS total_time_spent,
    ROUND(
        ((SUM(time_spent) / SUM(SUM(time_spent)) OVER()) * 100)::numeric, 
        2
    ) AS time_percentage
FROM 
    work_time
WHERE 
    date BETWEEN $start_date AND $end_date
GROUP BY 
    task_id
ORDER BY 
    time_percentage DESC;
