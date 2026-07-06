SELECT 
  id, 
  active,
  project, 
  title, 
  detail, 
  priority, 
FROM daily_tasks
WHERE id = $id;
