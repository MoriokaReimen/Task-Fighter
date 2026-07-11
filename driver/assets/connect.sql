CREATE SEQUENCE IF NOT EXISTS tasks_id_seq START 1;
CREATE TABLE IF NOT EXISTS tasks (
    id          INTEGER PRIMARY KEY DEFAULT nextval('tasks_id_seq'),
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
-- DailyTask
CREATE SEQUENCE IF NOT EXISTS daily_tasks_id_seq START 1;
CREATE TABLE IF NOT EXISTS daily_tasks (
    id          INTEGER PRIMARY KEY DEFAULT nextval('daily_tasks_id_seq'),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1,
);
-- WeeklyTask
CREATE SEQUENCE IF NOT EXISTS weekly_tasks_id_seq START 1;
CREATE TABLE IF NOT EXISTS weekly_tasks (
    id          INTEGER PRIMARY KEY DEFAULT nextval('weekly_tasks_id_seq'),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1,
    start_day   UTINYINT NOT NULL DEFAULT 1,
    end_day   UTINYINT NOT NULL DEFAULT 1,
);
-- MonthlyTask
CREATE SEQUENCE IF NOT EXISTS monthly_tasks_id_seq START 1;
CREATE TABLE IF NOT EXISTS monthly_tasks (
    id          INTEGER PRIMARY KEY DEFAULT nextval('monthly_tasks_id_seq'),
    active      BOOL NOT NULL DEFAULT true,
    project     VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    detail      VARCHAR NOT NULL,
    priority    UTINYINT NOT NULL DEFAULT 1,
    start_day   UTINYINT NOT NULL DEFAULT 1,
    end_day   UTINYINT NOT NULL DEFAULT 1,
);
-- Relation
CREATE TABLE IF NOT EXISTS relation (
    parent_id INTEGER,
    child_id INTEGER,
    PRIMARY KEY (parent_id, child_id)
);
-- WorkTime
CREATE SEQUENCE IF NOT EXISTS seq_work_time_id;
CREATE TABLE IF NOT EXISTS work_time (
    id INTEGER PRIMARY KEY DEFAULT nextval('seq_work_time_id'),
    task_id INTEGER NOT NULL,
    date DATE NOT NULL,
    time_spent REAL NOT NULL DEFAULT 0.0
);
-- Config
CREATE TYPE IF NOT EXISTS color_scheme_enum AS ENUM ('LightBlue');
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY,
    color_scheme color_scheme_enum NOT NULL
);

