SELECT 
    id, 
    active, 
    status, 
    project, 
    title, 
    detail, 
    start_date::TEXT, 
    due_date::TEXT, 
    priority, 
    progress, 
    time_spent 
FROM tasks 
WHERE 
    (
        regexp_matches(title, ?1, 'i') 
        OR regexp_matches(detail, ?1, 'i') 
        OR regexp_matches(project, ?1, 'i')
    )
    -- ?2 が false、あるいは active が true の場合にマッチさせる（統合ロジック）
    AND (?2 = false OR active = true)
ORDER BY priority DESC;
