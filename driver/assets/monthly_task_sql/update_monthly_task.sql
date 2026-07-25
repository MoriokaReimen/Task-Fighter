UPDATE monthly_tasks
SET
  active     = $active,
  project    = $project,
  title      = $title,
  detail     = $detail,
  priority   = $priority,
  start_day  = $start_day,
  due_day    = $due_day
WHERE uuid = $uuid;
