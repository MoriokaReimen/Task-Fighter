SELECT 
  id, 
  parent_id,
  child_id
FROM relation 
WHERE parent_id = $parent_id;

