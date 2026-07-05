INSERT INTO tasks
  (active, status, project, title, detail, start_date, due_date, priority, progress, time_spent, entry_date)
VALUES
  (:active, :status, :project, :title, :detail, :start_date,
  :due_date, :priority, :progress, :time_spent, :entry_date);
