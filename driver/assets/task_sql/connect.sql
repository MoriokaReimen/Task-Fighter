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
