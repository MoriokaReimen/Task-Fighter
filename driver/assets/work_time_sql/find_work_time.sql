SELECT
id,
task_id,
date::VARCHAR AS date, -- ::VARCHAR を追加
time_spent 
FROM work_time 
WHERE task_id = $task_id AND date = $date;
