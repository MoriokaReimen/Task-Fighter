SELECT id, active, status, project, title, detail, start_date::VARCHAR, due_date::VARCHAR, priority, progress, time_spent FROM tasks WHERE active = true ORDER BY priority DESC
