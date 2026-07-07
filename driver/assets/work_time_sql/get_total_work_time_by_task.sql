SELECT 
    COALESCE(SUM(time_spent), 0.0) AS total_time
FROM 
    work_time 
WHERE 
    task_id = $task_id;
