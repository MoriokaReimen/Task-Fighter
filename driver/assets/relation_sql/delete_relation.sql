DELETE FROM relation 
WHERE parent_id = $parent_id AND child_id = $child_id;
