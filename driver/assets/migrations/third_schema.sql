-- recreate tasks table
CREATE TABLE tasks_new (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    active      BOOL NOT NULL DEFAULT true,
    status      UTINYINT NOT NULL DEFAULT 0,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    start_date  DATE NOT NULL,
    due_date    DATE NOT NULL,
    entry_date  DATE NOT NULL,
    end_date    DATE,
    priority    UTINYINT NOT NULL DEFAULT 1,
    progress    REAL NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
    time_spent  REAL NOT NULL DEFAULT 0.0
);
INSERT INTO tasks_new(
  active,
  status,
  project,
  title,
  detail,
  start_date,
  due_date,
  entry_date,
  end_date,
  priority,
  progress,
  time_spent
) SELECT 
  active,
  status,
  project,
  title,
  detail,
  start_date,
  due_date,
  entry_date,
  end_date,
  priority,
  progress,
  time_spent
FROM tasks;
DROP TABLE tasks;
DROP SEQUENCE tasks_id_seq;
ALTER TABLE tasks_new RENAME TO tasks;

-- recreate daily tasks
CREATE TABLE daily_tasks_new (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1
);
INSERT INTO daily_tasks_new(
  active,
  project,
  title,
  detail,
  priority
) SELECT 
  active,
  project,
  title,
  detail,
  priority
FROM daily_tasks;
DROP TABLE daily_tasks;
DROP SEQUENCE daily_tasks_id_seq;
ALTER TABLE daily_tasks_new RENAME TO daily_tasks;

-- recreate weekly tasks
CREATE TABLE weekly_tasks_new (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1,
    start_day   UTINYINT NOT NULL DEFAULT 1,
    due_day     UTINYINT NOT NULL DEFAULT 1
);
INSERT INTO weekly_tasks_new(
  active,
  project,
  title,
  detail,
  priority,
  start_day,
  due_day
) SELECT 
  active,
  project,
  title,
  detail,
  priority,
  start_day,
  due_day
FROM weekly_tasks;
DROP TABLE weekly_tasks;
DROP SEQUENCE weekly_tasks_id_seq;
ALTER TABLE weekly_tasks_new RENAME TO weekly_tasks;

-- recreate monthly tasks
CREATE TABLE monthly_tasks_new (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1,
    start_day   UTINYINT NOT NULL DEFAULT 1,
    due_day     UTINYINT NOT NULL DEFAULT 1
);
INSERT INTO monthly_tasks_new(
  active,
  project,
  title,
  detail,
  priority,
  start_day,
  due_day
) SELECT 
  active,
  project,
  title,
  detail,
  priority,
  start_day,
  due_day
FROM monthly_tasks;
DROP TABLE monthly_tasks;
DROP SEQUENCE monthly_tasks_id_seq;
ALTER TABLE monthly_tasks_new RENAME TO monthly_tasks;
