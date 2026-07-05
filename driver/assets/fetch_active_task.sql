SELECT 
  id, 
  active, 
  status, 
  project, 
  title, 
  detail, 
  start_date::VARCHAR AS start_date,  -- 👈 「AS カラム名」を追加
  due_date::VARCHAR AS due_date,      -- 👈 追加
  priority, 
  progress, 
  time_spent, 
  entry_date::VARCHAR AS entry_date,  -- 👈 追加
  end_date::VARCHAR AS end_date       -- 👈 追加
FROM tasks 
WHERE active = true 
ORDER BY priority DESC;
