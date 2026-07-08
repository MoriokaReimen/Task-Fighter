SELECT EXISTS (
    SELECT 1 
    FROM work_time 
    WHERE id = $id
);
