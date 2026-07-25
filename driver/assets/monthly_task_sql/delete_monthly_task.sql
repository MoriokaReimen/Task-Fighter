UPDATE monthly_tasks
SET
  active     = false
WHERE uuid = $uuid;
