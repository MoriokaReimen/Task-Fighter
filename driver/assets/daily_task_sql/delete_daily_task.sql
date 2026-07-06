UPDATE daily_tasks
SET
  active     = false
WHERE id = $id;
