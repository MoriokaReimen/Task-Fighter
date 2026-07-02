UPDATE tasks 
SET active = ?1, status = ?2, project = ?3, title = ?4, detail = ?5, start_date = ?6, due_date = ?7, priority = ?8, progress = ?9, time_spent = ?10 
WHERE id = ?11
