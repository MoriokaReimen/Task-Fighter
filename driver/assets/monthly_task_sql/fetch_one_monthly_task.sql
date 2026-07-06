SELECT 
  id, 
  active,
  project, 
  title, 
  detail, 
  priority, 
  start_day,
  due_day
FROM monthly_tasks
WHERE id = $id;
