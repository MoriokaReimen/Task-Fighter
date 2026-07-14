SELECT 
  parent_id,
  child_id
FROM relation 
WHERE (child_id = $task_id) OR (parent_id = $task_id);

