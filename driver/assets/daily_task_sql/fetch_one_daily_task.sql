SELECT 
  uuid, 
  active,
  project, 
  title, 
  detail, 
  priority 
FROM daily_tasks
WHERE uuid = $uuid;
