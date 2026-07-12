UPDATE work_time
SET
  task_id     = $task_id,
  date        = $date,
  time_spent  = $time_spent
WHERE id = $id;
