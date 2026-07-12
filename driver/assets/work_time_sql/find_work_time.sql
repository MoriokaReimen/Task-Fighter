SELECT
id,
task_id,
date::VARCHAR AS date,
time_spent 
FROM work_time 
WHERE task_id = $task_id AND date = $date;
