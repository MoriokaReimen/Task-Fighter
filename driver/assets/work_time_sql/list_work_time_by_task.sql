SELECT id, task_id, date, time_spent 
FROM work_time 
WHERE task_id = $task_id 
ORDER BY date DESC
