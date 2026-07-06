UPDATE weekly_tasks
SET
  active     = false
WHERE id = $id;
