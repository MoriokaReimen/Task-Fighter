UPDATE tasks
SET
  active     = $active,
  status     = $status,
  project    = $project,
  title      = $title,
  detail     = $detail,
  start_date = $start_date,
  due_date   = $due_date,
  priority   = $priority,
  progress   = $progress,
  time_spent = $time_spent,
  entry_date = $entry_date,
  end_date   = $end_date
WHERE id = $id;
