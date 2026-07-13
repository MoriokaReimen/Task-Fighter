UPDATE daily_tasks
SET
  active     = $active,
  project    = $project,
  title      = $title,
  detail     = $detail,
  priority   = $priority
WHERE id = $id;
