UPDATE daily_tasks
SET
  active     = $active,
  project    = $project,
  title      = $title,
  detail     = $detail,
  priority   = $priority
WHERE uuid = $uuid;
