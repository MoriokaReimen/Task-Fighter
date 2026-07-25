UPDATE daily_tasks
SET
  active     = false
WHERE uuid = $uuid;
