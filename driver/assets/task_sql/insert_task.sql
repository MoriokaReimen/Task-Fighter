INSERT INTO tasks
  (uuid, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent, entry_date)
VALUES
  ($uuid, $active, $status, $project, $title, $detail, $start_date,
  $due_date, $priority, $progress, $time_spent, $entry_date);
