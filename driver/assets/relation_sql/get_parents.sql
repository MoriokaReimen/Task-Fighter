SELECT 
  id, 
  parent_id,
  child_id 
FROM relation 
WHERE child_id = $child_id;

