UPDATE monthly_tasks
SET
  active     = false
WHERE id = $id;
