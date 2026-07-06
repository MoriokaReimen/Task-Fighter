SELECT 
  id, 
  active,
  project, 
  title, 
  detail, 
  priority, 
  start_day,
  due_day
FROM weekly_tasks
WHERE id = $id;
