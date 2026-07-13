SELECT
    id,
    active,
    status,
    project,
    title,
    detail,
    start_date::VARCHAR AS start_date,
    due_date::VARCHAR AS due_date,
    priority,
    progress,
    time_spent,
    entry_date::VARCHAR AS entry_date,
    end_date::VARCHAR AS end_date
FROM tasks
WHERE active = true
ORDER BY priority DESC;
