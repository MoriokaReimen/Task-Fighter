UPDATE weekly_tasks
SET
  active     = false
WHERE uuid = $uuid;
